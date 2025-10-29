//! Initial data population API
//!
//! This file contains ONLY initial data population logic following Single Responsibility Principle.
//! System templates (workflows and prompts) are embedded into the binary at compile time.

use chrono::Utc;
use tracing::{error, info};

use crate::embedded_data;
use crate::store_singleton;

/// Populate initial workflows from embedded templates
/// Only runs if workflows table is empty (first run)
pub async fn populate_initial_workflows() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    // Check if we already have workflows
    let existing_workflows = store.list_workflows().await?;
    if !existing_workflows.is_empty() {
        info!("Workflows already exist, skipping initial population");
        return Ok(());
    }

    let mut loaded_count = 0;

    // Load workflows from embedded data
    for (filename, content) in embedded_data::get_default_workflows() {
        let file_data: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse JSON in {}: {}", filename, e);
            format!("Failed to parse JSON: {}", e)
        })?;

        // Extract fields from the file structure
        let id = file_data["id"]
            .as_str()
            .ok_or_else(|| format!("Missing 'id' field in {}", filename))?
            .to_string();
        let name = file_data["name"]
            .as_str()
            .ok_or_else(|| format!("Missing 'name' field in {}", filename))?
            .to_string();
        let description = file_data["description"].as_str().map(|s| s.to_string());

        // Get the content field and serialize it back to JSON string
        let content = file_data["content"].clone();
        let content_str = serde_json::to_string(&content).map_err(|e| {
            error!("Failed to serialize content: {}", e);
            format!("Failed to serialize content: {}", e)
        })?;

        // Create WorkflowDefinition (no special system type)
        let workflow = s_e_e_persistence::WorkflowDefinition {
            id: id.clone(),
            name,
            description,
            content: content_str,
            is_default: true,
            is_edited: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Validate
        workflow.validate()?;

        // Save to database
        store.save_workflow(&workflow).await?;
        loaded_count += 1;

        info!("Loaded initial workflow '{}'", id);
    }

    info!("Loaded {} initial workflows", loaded_count);
    Ok(())
}

/// Populate initial prompts from embedded templates
/// Only runs if prompts table is empty (first run)
pub async fn populate_initial_prompts() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    // Check if we already have prompts
    let existing_prompts = store.list_prompts().await?;
    if !existing_prompts.is_empty() {
        info!("Prompts already exist, skipping initial population");
        return Ok(());
    }

    let mut loaded_count = 0;

    // Load prompts from embedded data
    for (filename, content) in embedded_data::get_default_prompts() {
        let file_data: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse JSON in {}: {}", filename, e);
            format!("Failed to parse JSON: {}", e)
        })?;

        // Extract fields
        let id = file_data["id"]
            .as_str()
            .ok_or_else(|| format!("Missing 'id' field in {}", filename))?
            .to_string();
        let name = file_data["name"]
            .as_str()
            .ok_or_else(|| format!("Missing 'name' field in {}", filename))?
            .to_string();
        let content_str = file_data["content"]
            .as_str()
            .ok_or_else(|| format!("Missing 'content' field in {}", filename))?
            .to_string();

        // Create Prompt (no special system type)
        let prompt = s_e_e_persistence::Prompt {
            id: id.clone(),
            name,
            content: content_str,
            created_at: Utc::now(),
        };

        // Validate
        prompt.validate()?;

        // Save to database
        store.save_prompt(&prompt).await?;
        loaded_count += 1;

        info!("Loaded initial prompt '{}'", id);
    }

    info!("Loaded {} initial prompts", loaded_count);
    Ok(())
}

/// Audit and fix stuck workflow executions at startup
/// Marks any workflows in "running" state as "failed" since they cannot resume
pub async fn audit_stuck_workflows() -> Result<(), String> {
    info!("Auditing workflow executions for stuck processes...");

    let store = store_singleton::get_global_store()?;

    // Get all workflow executions
    let executions = store.list_workflow_executions().await?;

    let mut fixed_count = 0;

    // Find workflows stuck in "running" state
    for execution in executions {
        if matches!(
            execution.status,
            s_e_e_persistence::WorkflowExecutionStatus::Running
        ) {
            info!(
                "Found stuck workflow execution: {} ({})",
                execution.id, execution.workflow_name
            );

            // Update to failed state
            let mut fixed_execution = execution.clone();
            fixed_execution.status = s_e_e_persistence::WorkflowExecutionStatus::Failed;
            fixed_execution.completed_at = Some(Utc::now());
            fixed_execution.errors =
                vec!["Workflow was interrupted by application shutdown".to_string()];

            // Save updated execution
            store.save_workflow_execution(fixed_execution).await?;
            fixed_count += 1;

            info!(
                "Marked workflow execution {} as failed due to interruption",
                execution.id
            );
        }
    }

    if fixed_count > 0 {
        info!(
            "✓ Audited and fixed {} stuck workflow execution(s)",
            fixed_count
        );
    } else {
        info!("✓ No stuck workflow executions found");
    }

    Ok(())
}

/// Populate initial data (workflows and prompts)
/// Only runs on first startup when tables are empty
pub async fn populate_initial_data() -> Result<(), String> {
    info!("Populating initial data...");

    // First, audit and fix any stuck workflows from previous session
    audit_stuck_workflows().await?;

    // Populate from files to database if empty
    populate_initial_workflows().await?;
    populate_initial_prompts().await?;

    info!("✓ Initial data population complete");
    Ok(())
}
