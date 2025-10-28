use crate::queries::{GetRunningWorkflows, GetWorkflowHistory};
use dioxus_query::prelude::*;
use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

pub fn use_execution_list() -> (
    Result<Vec<WorkflowExecutionSummary>, String>,
    Result<Vec<WorkflowMetadata>, String>,
) {
    let history_query = use_query(Query::new((), GetWorkflowHistory));
    let history_result = match history_query.suspend() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e.to_string()),
    };

    let running_query = use_query(Query::new((), GetRunningWorkflows));
    let running_result = match running_query.suspend() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e.to_string()),
    };

    (history_result, running_result)
}
