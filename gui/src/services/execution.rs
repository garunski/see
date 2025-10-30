use s_e_e_core::{TaskExecution, WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Database not available: {0}")]
    DatabaseUnavailable(String),
    #[error("Failed to fetch workflow executions: {0}")]
    FetchExecutionsFailed(String),
    #[error("Failed to fetch running workflows: {0}")]
    FetchRunningWorkflowsFailed(String),
    #[error("Failed to fetch workflow execution: {0}")]
    FetchWorkflowExecutionFailed(String),
    #[error("Failed to fetch task details: {0}")]
    FetchTaskDetailsFailed(String),
    #[error("Failed to delete workflow execution: {0}")]
    DeleteExecutionFailed(String),
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

        if !running.is_empty() {
            tracing::trace!(
                running_count = running.len(),
                "Filtered {} running workflows",
                running.len()
            );
        }

        Ok(running)
    }

    pub async fn fetch_workflow_execution(
        execution_id: &str,
    ) -> Result<WorkflowExecution, ExecutionError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| ExecutionError::DatabaseUnavailable(e.to_string()))?;

        store
            .get_workflow_with_tasks(execution_id)
            .await
            .map_err(|e| ExecutionError::FetchWorkflowExecutionFailed(e.to_string()))
    }

    pub async fn fetch_task_details(
        execution_id: &str,
        task_id: &str,
    ) -> Result<Option<TaskExecution>, ExecutionError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| ExecutionError::DatabaseUnavailable(e.to_string()))?;

        let execution = store
            .get_workflow_with_tasks(execution_id)
            .await
            .map_err(|e| ExecutionError::FetchTaskDetailsFailed(e.to_string()))?;

        let task = execution.tasks.into_iter().find(|t| t.id == task_id);

        Ok(task)
    }

    pub async fn delete_workflow_execution(execution_id: &str) -> Result<(), ExecutionError> {
        s_e_e_core::delete_workflow_execution(execution_id)
            .await
            .map_err(|e| ExecutionError::DeleteExecutionFailed(e.to_string()))
    }
}
