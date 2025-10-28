use crate::pages::executions::details::hooks::use_workflow_execution;
use crate::pages::executions::details_2::components::WorkflowFlowGraph;
use dioxus::prelude::*;

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let (execution, loading, error) = use_workflow_execution(id.clone());

    rsx! {
        div {
            if loading() {
                div { class: "text-center text-zinc-500 dark:text-zinc-400", "Loading..." }
            }

            if let Some(err) = error() {
                div { class: "text-center text-red-600 dark:text-red-400", "Error: {err}" }
            }

            if let Some(exec) = execution() {
                WorkflowFlowGraph {
                    snapshot: exec.workflow_snapshot.clone(),
                    tasks: exec.tasks.clone()
                }
            }
        }
    }
}
