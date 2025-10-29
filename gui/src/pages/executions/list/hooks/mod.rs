use crate::queries::{use_running_workflows_query, use_workflow_executions_query};
use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

pub fn use_execution_list() -> (
    Result<Vec<WorkflowExecutionSummary>, String>,
    Result<Vec<WorkflowMetadata>, String>,
) {
    let (executions_state, _) = use_workflow_executions_query();
    let (running_state, _) = use_running_workflows_query();

    let executions_result = if executions_state.is_loading {
        Ok(vec![])
    } else if executions_state.is_error {
        Err(executions_state
            .error
            .clone()
            .unwrap_or_else(|| "Failed to load executions".to_string()))
    } else {
        Ok(executions_state.data.clone().unwrap_or_default())
    };

    let running_result = if running_state.is_loading {
        Ok(vec![])
    } else if running_state.is_error {
        Err(running_state
            .error
            .clone()
            .unwrap_or_else(|| "Failed to load running workflows".to_string()))
    } else {
        Ok(running_state.data.clone().unwrap_or_default())
    };

    (executions_result, running_result)
}
