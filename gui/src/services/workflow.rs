use s_e_e_core::{
    errors::CoreError, execute_workflow_by_id, OutputCallback, WorkflowDefinition, WorkflowResult,
};
use std::fs;

pub async fn run_workflow_by_id(
    workflow_id: String,
    output: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow_by_id(&workflow_id, output).await
}

pub fn read_and_parse_workflow_file(file_path: String) -> Result<WorkflowDefinition, String> {
    // Read file content
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

    // Parse JSON to extract name
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in workflow file: {}", e))?;

    let _workflow_name = json_value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Workflow")
        .to_string();

    // Generate unique ID
    let workflow_id = format!("custom-workflow-{}", chrono::Utc::now().timestamp());

    // Create WorkflowDefinition
    Ok(WorkflowDefinition {
        id: workflow_id,
        name: "Imported Workflow".to_string(),
        description: Some("Imported from file".to_string()),
        content,
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}
