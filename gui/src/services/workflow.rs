use s_e_e_core::WorkflowDefinition;
use std::fs;

#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Database not available: {0}")]
    DatabaseUnavailable(String),
    #[error("Failed to fetch workflows: {0}")]
    FetchWorkflowsFailed(String),
    #[error("Failed to fetch workflow: {0}")]
    FetchWorkflowFailed(String),
    #[error("Failed to create workflow: {0}")]
    CreateWorkflowFailed(String),
}

pub struct WorkflowService;

impl WorkflowService {
    pub async fn fetch_workflows() -> Result<Vec<WorkflowDefinition>, WorkflowError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| WorkflowError::DatabaseUnavailable(e.to_string()))?;

        store
            .list_workflows()
            .await
            .map_err(|e| WorkflowError::FetchWorkflowsFailed(e.to_string()))
    }

    pub async fn fetch_workflow(id: &str) -> Result<Option<WorkflowDefinition>, WorkflowError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| WorkflowError::DatabaseUnavailable(e.to_string()))?;

        store
            .get_workflow(id)
            .await
            .map_err(|e| WorkflowError::FetchWorkflowFailed(e.to_string()))
    }

    pub async fn create_workflow(workflow: WorkflowDefinition) -> Result<(), WorkflowError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| WorkflowError::DatabaseUnavailable(e.to_string()))?;

        store
            .save_workflow(&workflow)
            .await
            .map_err(|e| WorkflowError::CreateWorkflowFailed(e.to_string()))
    }
}

pub fn read_and_parse_workflow_file(file_path: String) -> Result<WorkflowDefinition, String> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

    s_e_e_core::validate_workflow_json(&content)
        .map_err(|e| format!("Validation failed:\n{}", e))?;

    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in workflow file: {}", e))?;

    let workflow_name = json_value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Workflow")
        .to_string();

    let workflow_id = format!("custom-workflow-{}", chrono::Utc::now().timestamp());

    Ok(WorkflowDefinition {
        id: workflow_id,
        name: workflow_name,
        description: Some("Imported from file".to_string()),
        content,
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}
