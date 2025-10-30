use chrono::Utc;
use tracing::{error, info};

use crate::embedded_data;
use crate::store_singleton;

pub async fn populate_initial_workflows() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    let existing_workflows = store.list_workflows().await?;
    if !existing_workflows.is_empty() {
        info!("Workflows already exist, skipping initial population");
        return Ok(());
    }

    let mut loaded_count = 0;

    for (filename, content) in embedded_data::get_default_workflows() {
        let file_data: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse JSON in {}: {}", filename, e);
            format!("Failed to parse JSON: {}", e)
        })?;

        let id = file_data["id"]
            .as_str()
            .ok_or_else(|| format!("Missing 'id' field in {}", filename))?
            .to_string();
        let name = file_data["name"]
            .as_str()
            .ok_or_else(|| format!("Missing 'name' field in {}", filename))?
            .to_string();
        let description = file_data["description"].as_str().map(|s| s.to_string());

        let content = file_data["content"].clone();
        let content_str = serde_json::to_string(&content).map_err(|e| {
            error!("Failed to serialize content: {}", e);
            format!("Failed to serialize content: {}", e)
        })?;

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

        workflow.validate()?;

        store.save_workflow(&workflow).await?;
        loaded_count += 1;

        info!("Loaded initial workflow '{}'", id);
    }

    info!("Loaded {} initial workflows", loaded_count);
    Ok(())
}

pub async fn populate_initial_prompts() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    let existing_prompts = store.list_prompts().await?;
    if !existing_prompts.is_empty() {
        info!("Prompts already exist, skipping initial population");
        return Ok(());
    }

    let mut loaded_count = 0;

    for (filename, content) in embedded_data::get_default_prompts() {
        let file_data: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse JSON in {}: {}", filename, e);
            format!("Failed to parse JSON: {}", e)
        })?;

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

        let prompt = s_e_e_persistence::Prompt {
            id: id.clone(),
            name,
            content: content_str,
            created_at: Utc::now(),
        };

        prompt.validate()?;

        store.save_prompt(&prompt).await?;
        loaded_count += 1;

        info!("Loaded initial prompt '{}'", id);
    }

    info!("Loaded {} initial prompts", loaded_count);
    Ok(())
}

pub async fn audit_stuck_workflows() -> Result<(), String> {
    info!("Auditing workflow executions for stuck processes...");

    let store = store_singleton::get_global_store()?;

    let executions = store.list_workflow_executions().await?;

    let mut fixed_count = 0;

    for execution in executions {
        if matches!(
            execution.status,
            s_e_e_persistence::WorkflowExecutionStatus::Running
        ) {
            info!(
                "Found stuck workflow execution: {} ({})",
                execution.id, execution.workflow_name
            );

            let mut fixed_execution = execution.clone();
            fixed_execution.status = s_e_e_persistence::WorkflowExecutionStatus::Failed;
            fixed_execution.completed_at = Some(Utc::now());
            fixed_execution.errors =
                vec!["Workflow was interrupted by application shutdown".to_string()];

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

pub async fn populate_initial_data() -> Result<(), String> {
    info!("Populating initial data...");

    audit_stuck_workflows().await?;

    populate_initial_workflows().await?;
    populate_initial_prompts().await?;

    info!("✓ Initial data population complete");
    Ok(())
}
