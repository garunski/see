use s_e_e_core::persistence::models::WorkflowStatus;
use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("Database not available: {0}")]
    DatabaseUnavailable(String),
    #[error("Failed to fetch workflow executions: {0}")]
    FetchExecutionsFailed(String),
    #[error("Failed to fetch running workflows: {0}")]
    FetchRunningWorkflowsFailed(String),
    #[error("Failed to delete execution: {0}")]
    DeleteExecutionFailed(String),
    #[error("Failed to delete running workflow: {0}")]
    DeleteRunningWorkflowFailed(String),
}

pub struct HistoryService;

impl HistoryService {
    pub async fn fetch_workflow_executions(
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        store
            .list_workflow_executions(limit)
            .await
            .map_err(|e| HistoryError::FetchExecutionsFailed(e.to_string()))
    }

    pub async fn fetch_running_workflows(
        limit: usize,
    ) -> Result<Vec<WorkflowMetadata>, HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        let metadata = store
            .list_workflow_metadata(limit)
            .await
            .map_err(|e| HistoryError::FetchRunningWorkflowsFailed(e.to_string()))?;

        let running: Vec<_> = metadata
            .into_iter()
            .filter(|m| m.status == WorkflowStatus::Running)
            .collect();

        Ok(running)
    }

    pub async fn refresh_all(
        limit: usize,
    ) -> Result<(Vec<WorkflowExecutionSummary>, Vec<WorkflowMetadata>), HistoryError> {
        let (executions, running) = tokio::try_join!(
            Self::fetch_workflow_executions(limit),
            Self::fetch_running_workflows(limit)
        )?;

        Ok((executions, running))
    }

    pub async fn delete_execution(id: &str) -> Result<(), HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        store
            .delete_workflow_execution(id)
            .await
            .map_err(|e| HistoryError::DeleteExecutionFailed(e.to_string()))
    }

    pub async fn delete_running_workflow(id: &str) -> Result<(), HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        store
            .delete_workflow_metadata_and_tasks(id)
            .await
            .map_err(|e| HistoryError::DeleteRunningWorkflowFailed(e.to_string()))
    }
}
