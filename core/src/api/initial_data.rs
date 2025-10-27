//! Initial data population API
//!
//! This file contains ONLY initial data population logic following Single Responsibility Principle.

use chrono::Utc;
use tracing::{error, info};

use crate::store_singleton;

/// Populate initial workflows from the /system/workflows directory
/// Only runs if workflows table is empty (first run)
pub async fn populate_initial_workflows() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    // Check if we already have workflows
    let existing_workflows = store.list_workflows().await?;
    if !existing_workflows.is_empty() {
        info!("Workflows already exist, skipping initial population");
        return Ok(());
    }

    // Get the current working directory (workspace root)
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let system_dir = current_dir.join("system/workflows");

    if !system_dir.exists() {
        info!(
            "System workflows directory not found at {:?}, skipping",
            system_dir
        );
        return Ok(());
    }

    // Read all JSON files in the directory
    let entries = std::fs::read_dir(&system_dir).map_err(|e| {
        error!("Failed to read system workflows directory: {}", e);
        format!("Failed to read directory: {}", e)
    })?;

    let mut loaded_count = 0;

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            format!("Failed to read entry: {}", e)
        })?;

        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "json" {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    error!("Failed to read file {:?}: {}", path, e);
                    format!("Failed to read file: {}", e)
                })?;

                let file_data: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                    error!("Failed to parse JSON in {:?}: {}", path, e);
                    format!("Failed to parse JSON: {}", e)
                })?;

                // Extract fields from the file structure
                let id = file_data["id"]
                    .as_str()
                    .ok_or_else(|| "Missing 'id' field".to_string())?
                    .to_string();
                let name = file_data["name"]
                    .as_str()
                    .ok_or_else(|| "Missing 'name' field".to_string())?
                    .to_string();
                let description = file_data["description"].as_str().map(|s| s.to_string());

                // Get the content field and serialize it back to JSON string
                let content = file_data["content"].clone();
                let content_str = serde_json::to_string(&content).map_err(|e| {
                    error!("Failed to serialize content: {}", e);
                    format!("Failed to serialize content: {}", e)
                })?;

                // Create WorkflowDefinition (no special system type)
                let workflow = persistence::WorkflowDefinition {
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
        }
    }

    info!("Loaded {} initial workflows", loaded_count);
    Ok(())
}

/// Populate initial prompts from the /system/prompts directory
/// Only runs if prompts table is empty (first run)
pub async fn populate_initial_prompts() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    // Check if we already have prompts
    let existing_prompts = store.list_prompts().await?;
    if !existing_prompts.is_empty() {
        info!("Prompts already exist, skipping initial population");
        return Ok(());
    }

    // Get the current working directory (workspace root)
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let system_dir = current_dir.join("system/prompts");

    if !system_dir.exists() {
        info!(
            "System prompts directory not found at {:?}, skipping",
            system_dir
        );
        return Ok(());
    }

    // Read all JSON files in the directory
    let entries = std::fs::read_dir(&system_dir).map_err(|e| {
        error!("Failed to read system prompts directory: {}", e);
        format!("Failed to read directory: {}", e)
    })?;

    let mut loaded_count = 0;

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            format!("Failed to read entry: {}", e)
        })?;

        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "json" {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    error!("Failed to read file {:?}: {}", path, e);
                    format!("Failed to read file: {}", e)
                })?;

                let file_data: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                    error!("Failed to parse JSON in {:?}: {}", path, e);
                    format!("Failed to parse JSON: {}", e)
                })?;

                // Extract fields
                let id = file_data["id"]
                    .as_str()
                    .ok_or_else(|| "Missing 'id' field".to_string())?
                    .to_string();
                let name = file_data["name"]
                    .as_str()
                    .ok_or_else(|| "Missing 'name' field".to_string())?
                    .to_string();
                let description = file_data["description"].as_str().map(|s| s.to_string());
                let content_str = file_data["content"]
                    .as_str()
                    .ok_or_else(|| "Missing 'content' field".to_string())?
                    .to_string();
                let template = file_data["template"]
                    .as_str()
                    .ok_or_else(|| "Missing 'template' field".to_string())?
                    .to_string();

                // Extract variables array
                let variables = file_data["variables"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                // Extract tags array
                let tags = file_data["tags"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let metadata = file_data.get("metadata").cloned().unwrap_or_default();

                // Create Prompt (no special system type)
                let prompt = persistence::Prompt {
                    id: id.clone(),
                    name,
                    content: content_str,
                    description,
                    template,
                    variables,
                    tags,
                    metadata,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // Validate
                prompt.validate()?;

                // Save to database
                store.save_prompt(&prompt).await?;
                loaded_count += 1;

                info!("Loaded initial prompt '{}'", id);
            }
        }
    }

    info!("Loaded {} initial prompts", loaded_count);
    Ok(())
}

/// Populate initial data (workflows and prompts)
/// Only runs on first startup when tables are empty
pub async fn populate_initial_data() -> Result<(), String> {
    info!("Populating initial data...");

    // Populate from files to database if empty
    populate_initial_workflows().await?;
    populate_initial_prompts().await?;

    info!("✓ Initial data population complete");
    Ok(())
}
