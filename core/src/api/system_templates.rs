//! System templates API
//!
//! This file contains ONLY system template operations following Single Responsibility Principle.

use chrono::Utc;
use tracing::{error, info};

use crate::store_singleton;

/// Load system workflows from the /system/workflows directory
pub async fn load_system_workflows() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    // Get the current working directory (workspace root)
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let system_dir = current_dir.join("system/workflows");

    let system_dir_str = system_dir.to_string_lossy().to_string();

    if !system_dir.exists() {
        return Err(format!(
            "CRITICAL: System workflows directory not found at {:?}",
            system_dir
        ));
    }

    // Clear existing system workflows to allow updates
    store.clear_system_workflows().await?;

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
                let version = file_data["version"]
                    .as_str()
                    .ok_or_else(|| "Missing 'version' field".to_string())?
                    .to_string();

                // Get the content field and serialize it back to JSON string
                let content = file_data["content"].clone();
                let content_str = serde_json::to_string(&content).map_err(|e| {
                    error!("Failed to serialize content: {}", e);
                    format!("Failed to serialize content: {}", e)
                })?;

                // Create SystemWorkflow
                let system_workflow = persistence::SystemWorkflow {
                    id: id.clone(),
                    name,
                    description,
                    content: content_str,
                    version: version.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // Validate
                system_workflow.validate()?;

                // Save to database
                store.save_system_workflow(&system_workflow).await?;
                loaded_count += 1;

                info!("Loaded system workflow '{}' (version {})", id, version);
            }
        }
    }

    if loaded_count == 0 {
        return Err(format!(
            "CRITICAL: No system workflows loaded from {}",
            system_dir_str
        ));
    }

    info!("Loaded {} system workflows", loaded_count);
    Ok(())
}

/// Load system prompts from the /system/prompts directory
pub async fn load_system_prompts() -> Result<(), String> {
    let store = store_singleton::get_global_store()?;

    // Get the current working directory (workspace root)
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let system_dir = current_dir.join("system/prompts");

    let system_dir_str = system_dir.to_string_lossy().to_string();

    if !system_dir.exists() {
        return Err(format!(
            "CRITICAL: System prompts directory not found at {:?}",
            system_dir
        ));
    }

    // Clear existing system prompts to allow updates
    store.clear_system_prompts().await?;

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
                let version = file_data["version"]
                    .as_str()
                    .ok_or_else(|| "Missing 'version' field".to_string())?
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

                // Create SystemPrompt
                let system_prompt = persistence::SystemPrompt {
                    id: id.clone(),
                    name,
                    content: content_str,
                    description,
                    template,
                    variables,
                    tags,
                    metadata,
                    version: version.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // Validate
                system_prompt.validate()?;

                // Save to database
                store.save_system_prompt(&system_prompt).await?;
                loaded_count += 1;

                info!("Loaded system prompt '{}' (version {})", id, version);
            }
        }
    }

    if loaded_count == 0 {
        return Err(format!(
            "CRITICAL: No system prompts loaded from {}",
            system_dir_str
        ));
    }

    info!("Loaded {} system prompts", loaded_count);
    Ok(())
}

/// Clone a system workflow to create a user workflow
pub async fn clone_system_workflow(
    system_workflow_id: &str,
    new_name: Option<String>,
) -> Result<persistence::WorkflowDefinition, String> {
    let store = store_singleton::get_global_store()?;

    // Get the system workflow
    let system_workflow = store
        .get_system_workflow(system_workflow_id)
        .await?
        .ok_or_else(|| format!("System workflow '{}' not found", system_workflow_id))?;

    // Create a new user workflow based on it
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_name = new_name.unwrap_or_else(|| format!("Copy of {}", system_workflow.name));

    let user_workflow = persistence::WorkflowDefinition {
        id: new_id,
        name: new_name,
        description: system_workflow.description.clone(),
        content: system_workflow.content.clone(),
        is_default: false,
        is_edited: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Validate
    user_workflow.validate()?;

    // Save to database
    store.save_workflow(&user_workflow).await?;

    info!(
        "Cloned system workflow '{}' to user workflow '{}'",
        system_workflow_id, user_workflow.id
    );
    Ok(user_workflow)
}

/// Clone a system prompt to create a user prompt
pub async fn clone_system_prompt(
    system_prompt_id: &str,
    new_name: Option<String>,
) -> Result<persistence::UserPrompt, String> {
    let store = store_singleton::get_global_store()?;

    // Get the system prompt
    let system_prompt = store
        .get_system_prompt(system_prompt_id)
        .await?
        .ok_or_else(|| format!("System prompt '{}' not found", system_prompt_id))?;

    // Create a new user prompt based on it
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_name = new_name.unwrap_or_else(|| format!("Copy of {}", system_prompt.name));

    let user_prompt = persistence::UserPrompt {
        id: new_id,
        name: new_name,
        content: system_prompt.content.clone(),
        description: system_prompt.description.clone(),
        template: system_prompt.template.clone(),
        variables: system_prompt.variables.clone(),
        tags: system_prompt.tags.clone(),
        metadata: system_prompt.metadata.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Validate
    user_prompt.validate()?;

    // Save to database
    store.save_prompt(&user_prompt).await?;

    info!(
        "Cloned system prompt '{}' to user prompt '{}'",
        system_prompt_id, user_prompt.id
    );
    Ok(user_prompt)
}

/// Initialize and load all system templates
pub async fn load_all_system_templates() -> Result<(), String> {
    info!("Loading system templates...");

    // Load templates from files to database
    load_system_workflows().await?;
    load_system_prompts().await?;

    // Verify they actually loaded into database
    let store = store_singleton::get_global_store()?;
    let workflows = store.list_system_workflows().await?;
    let prompts = store.list_system_prompts().await?;

    if workflows.len() < 3 {
        return Err(format!(
            "CRITICAL: Expected at least 3 system workflows, found {}",
            workflows.len()
        ));
    }
    if prompts.len() < 3 {
        return Err(format!(
            "CRITICAL: Expected at least 3 system prompts, found {}",
            prompts.len()
        ));
    }

    info!(
        "✓ System templates loaded: {} workflows, {} prompts",
        workflows.len(),
        prompts.len()
    );
    Ok(())
}
