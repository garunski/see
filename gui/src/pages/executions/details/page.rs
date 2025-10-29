use crate::components::{IconButton, IconButtonSize, IconButtonVariant, PageHeader};
use crate::layout::router::Route;
use crate::pages::executions::details::components::WorkflowFlowGraph;
use crate::pages::executions::list::components::ExecutionDeleteDialog;
use crate::queries::{DeleteExecutionMutation, GetWorkflowExecution};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query, Mutation, Query};
use dioxus_router::prelude::use_navigator;

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let navigator = use_navigator();
    let mut show_delete_dialog = use_signal(|| false);
    let delete_mutation = use_mutation(Mutation::new(DeleteExecutionMutation));

    let query_result = use_query(
        Query::new(id.clone(), GetWorkflowExecution)
            .interval_time(std::time::Duration::from_secs(2)),
    );

    let execution = match query_result.suspend() {
        Ok(Ok(exec)) => Some(exec),
        _ => None,
    };

    let (execution_id, workflow_name) = if let Some(exec) = execution.as_ref() {
        (Some(exec.id.clone()), Some(exec.workflow_name.clone()))
    } else {
        (None, None)
    };

    rsx! {
        div { class: "space-y-6",
            if let Some(exec) = execution.as_ref() {
                PageHeader {
                    title: exec.workflow_name.clone(),
                    description: format!("Execution ID: {}", exec.id),
                    actions: Some(rsx! {
                        IconButton {
                            variant: IconButtonVariant::Danger,
                            size: IconButtonSize::Medium,
                            onclick: move |_| {
                                show_delete_dialog.set(true);
                            },
                            icon: Some("trash".to_string()),
                            icon_variant: "outline".to_string(),
                            "Delete"
                        }
                    }),
                }

                WorkflowFlowGraph {
                    snapshot: exec.workflow_snapshot.clone(),
                    tasks: exec.tasks.clone(),
                    execution_id: exec.id.clone()
                }

                if let (Some(exec_id), Some(workflow_name)) = (execution_id, workflow_name) {
                    ExecutionDeleteDialog {
                        show: show_delete_dialog(),
                        execution_id: exec_id.clone(),
                        workflow_name: workflow_name.clone(),
                        on_confirm: move |_| {
                            show_delete_dialog.set(false);
                            let execution_id = exec_id.clone();
                            let nav = navigator.clone();
                            let delete_mutation_clone = delete_mutation.clone();
                            spawn(async move {
                                let _reader = delete_mutation_clone.mutate_async(execution_id.clone()).await;
                                // Navigate back to execution list after deletion
                                nav.push(Route::ExecutionListPage {});
                            });
                        },
                        on_cancel: move |_| {
                            show_delete_dialog.set(false);
                        },
                    }
                }
            } else {
                PageHeader {
                    title: "Execution Details".to_string(),
                    description: "Loading execution...".to_string(),
                    actions: None,
                }
            }
        }
    }
}
