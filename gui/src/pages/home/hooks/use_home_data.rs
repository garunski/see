use crate::queries::{GetWorkflowExecutions, GetWorkflows};
use dioxus_query::prelude::*;
use s_e_e_core::{WorkflowDefinition, WorkflowExecutionSummary};

/// Hook to fetch workflows list
pub fn use_workflows_list() -> Result<Vec<WorkflowDefinition>, String> {
    let query_result = use_query(Query::new((), GetWorkflows))
        .suspend()
        .map_err(|_| String::from("Failed to initialize query"))?;

    match query_result {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::error!("Failed to load workflows: {}", e);
            Err(format!("Failed to load workflows: {}", e))
        }
    }
}

/// Hook to fetch workflow executions
pub fn use_workflow_executions() -> Result<Vec<WorkflowExecutionSummary>, String> {
    let query_result = use_query(Query::new((), GetWorkflowExecutions))
        .suspend()
        .map_err(|_| String::from("Failed to initialize query"))?;

    match query_result {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::error!("Failed to load workflow executions: {}", e);
            Err(format!("Failed to load workflow executions: {}", e))
        }
    }
}
