use crate::queries::GetWorkflow;
use dioxus_query::prelude::*;
use s_e_e_core::WorkflowDefinition;

/// Hook to fetch a single workflow by ID
pub fn use_workflow_visualizer(id: String) -> Result<Option<WorkflowDefinition>, String> {
    if id.is_empty() {
        return Ok(None);
    }

    let query_result = use_query(Query::new(id.clone(), GetWorkflow))
        .suspend()
        .map_err(|_| String::from("Failed to initialize query"))?;

    match query_result {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::error!("Failed to load workflow: {}", e);
            Err(format!("Failed to load workflow: {}", e))
        }
    }
}
