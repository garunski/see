use crate::queries::{GetRunningWorkflows, GetWorkflowExecutions};
use dioxus_query::prelude::*;
use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

pub fn use_execution_list() -> (
    Result<Vec<WorkflowExecutionSummary>, String>,
    Result<Vec<WorkflowMetadata>, String>,
) {
    let executions_query = use_query(
        Query::new((), GetWorkflowExecutions).interval_time(std::time::Duration::from_secs(1)),
    );
    let executions_result = match executions_query.suspend() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e.to_string()),
    };

    let running_query = use_query(
        Query::new((), GetRunningWorkflows).interval_time(std::time::Duration::from_secs(1)),
    );
    let running_result = match running_query.suspend() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e.to_string()),
    };

    (executions_result, running_result)
}
