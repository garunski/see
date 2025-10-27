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
    #[allow(dead_code)]
    DeleteExecutionFailed(String),
    #[error("Failed to delete running workflow: {0}")]
    #[allow(dead_code)]
    DeleteRunningWorkflowFailed(String),
}

pub struct HistoryService;

impl HistoryService {
    pub async fn fetch_workflow_executions(
        _limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        let executions = store
            .list_workflow_executions()
            .await
            .map_err(|e| HistoryError::FetchExecutionsFailed(e.to_string()))?;

        // Convert WorkflowExecution to WorkflowExecutionSummary
        let summaries = executions
            .into_iter()
            .map(|exec| WorkflowExecutionSummary {
                id: exec.id,
                workflow_name: exec.workflow_name,
                status: exec.status,
                created_at: exec.created_at,
                completed_at: exec.completed_at,
                task_count: exec.tasks.len(),
                timestamp: exec.timestamp,
            })
            .collect();

        Ok(summaries)
    }

    pub async fn fetch_running_workflows(
        _limit: usize,
    ) -> Result<Vec<WorkflowMetadata>, HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        let metadata = store
            .list_workflow_metadata()
            .await
            .map_err(|e| HistoryError::FetchRunningWorkflowsFailed(e.to_string()))?;

        // Filter for truly active running workflows
        // Exclude workflows that are in workflow_executions (which means they're waiting or completed)
        let all_execution_ids = store
            .list_workflow_executions()
            .await
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?
            .into_iter()
            .map(|exec| exec.id)
            .collect::<std::collections::HashSet<_>>();

        let running: Vec<_> = metadata
            .into_iter()
            .filter(|m| m.status == "running" && !all_execution_ids.contains(&m.id))
            .collect();

        tracing::trace!(
            running_count = running.len(),
            "Filtered {} running workflows",
            running.len()
        );

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

    #[allow(dead_code)]
    pub async fn delete_execution(id: &str) -> Result<(), HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        store
            .delete_workflow_execution(id)
            .await
            .map_err(|e| HistoryError::DeleteExecutionFailed(e.to_string()))
    }

    #[allow(dead_code)]
    pub async fn delete_running_workflow(id: &str) -> Result<(), HistoryError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| HistoryError::DatabaseUnavailable(e.to_string()))?;

        store
            .delete_workflow_metadata_and_tasks(id)
            .await
            .map_err(|e| HistoryError::DeleteRunningWorkflowFailed(e.to_string()))
    }
}
