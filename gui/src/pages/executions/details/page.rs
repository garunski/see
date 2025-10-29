use crate::components::{IconButton, IconButtonSize, IconButtonVariant, PageHeader};
use crate::layout::router::Route;
use crate::pages::executions::details::components::WorkflowFlowGraph;
use crate::pages::executions::list::components::ExecutionDeleteDialog;
use crate::queries::{use_delete_execution_mutation, use_workflow_execution_query};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let navigator = use_navigator();
    let mut show_delete_dialog = use_signal(|| false);
    let (_delete_state, delete_fn) = use_delete_execution_mutation();

    let (exec_state, _refetch) = use_workflow_execution_query(id.clone());

    let execution = if exec_state.is_loading {
        return rsx! {
            div { class: "flex items-center justify-center h-64",
                "Loading execution details..."
            }
        };
    } else if exec_state.is_error {
        None
    } else {
        exec_state.data.clone()
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
                            delete_fn(execution_id.clone());
                            // Navigate back to execution list after deletion
                            navigator.push(Route::ExecutionListPage {});
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
