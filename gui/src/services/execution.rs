use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Database not available: {0}")]
    DatabaseUnavailable(String),
    #[error("Failed to fetch workflow executions: {0}")]
    FetchExecutionsFailed(String),
    #[error("Failed to fetch running workflows: {0}")]
    FetchRunningWorkflowsFailed(String),
}

pub struct ExecutionService;

impl ExecutionService {
    pub async fn fetch_workflow_executions(
        _limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, ExecutionError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| ExecutionError::DatabaseUnavailable(e.to_string()))?;

        let executions = store
            .list_workflow_executions()
            .await
            .map_err(|e| ExecutionError::FetchExecutionsFailed(e.to_string()))?;

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
    ) -> Result<Vec<WorkflowMetadata>, ExecutionError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| ExecutionError::DatabaseUnavailable(e.to_string()))?;

        let metadata = store
            .list_workflow_metadata()
            .await
            .map_err(|e| ExecutionError::FetchRunningWorkflowsFailed(e.to_string()))?;

        // Filter for truly active running workflows
        // Exclude workflows that are in workflow_executions (which means they're waiting or completed)
        let all_execution_ids = store
            .list_workflow_executions()
            .await
            .map_err(|e| ExecutionError::DatabaseUnavailable(e.to_string()))?
            .into_iter()
            .map(|exec| exec.id)
            .collect::<std::collections::HashSet<_>>();

        let running: Vec<_> = metadata
            .into_iter()
            .filter(|m| m.status == "running" && !all_execution_ids.contains(&m.id))
            .collect();

        // Only log when there are actual running workflows to avoid spam
        if !running.is_empty() {
            tracing::trace!(
                running_count = running.len(),
                "Filtered {} running workflows",
                running.len()
            );
        }

        Ok(running)
    }
}
