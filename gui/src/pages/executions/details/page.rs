use crate::pages::executions::details::components::WorkflowFlowGraph;
use crate::queries::GetWorkflowExecution;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_query, Query};

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let query_result = use_query(
        Query::new(id.clone(), GetWorkflowExecution)
            .interval_time(std::time::Duration::from_secs(2)),
    );

    let execution = match query_result.suspend() {
        Ok(Ok(exec)) => Some(exec),
        _ => None,
    };

    rsx! {
        div {
            if let Some(exec) = execution.as_ref() {
                WorkflowFlowGraph {
                    snapshot: exec.workflow_snapshot.clone(),
                    tasks: exec.tasks.clone(),
                    execution_id: exec.id.clone()
                }
            }
        }
    }
}
