use crate::components::layout::ListItem;
use crate::components::{Badge, BadgeColor, IconButton, IconButtonSize, IconButtonVariant};
use crate::layout::router::Route;
use crate::pages::executions::list::components::ExecutionDeleteDialog;
use crate::queries::DeleteExecutionMutation;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, Mutation};
use dioxus_router::prelude::use_navigator;
use s_e_e_core::{WorkflowExecutionStatus, WorkflowExecutionSummary};

#[component]
pub fn ExecutionItem(execution: WorkflowExecutionSummary) -> Element {
    let navigator = use_navigator();
    let mut show_delete_dialog = use_signal(|| false);
    let delete_mutation = use_mutation(Mutation::new(DeleteExecutionMutation));

    let badge_color = match execution.status {
        WorkflowExecutionStatus::WaitingForInput => BadgeColor::Amber,
        WorkflowExecutionStatus::Complete => BadgeColor::Emerald,
        WorkflowExecutionStatus::Failed => BadgeColor::Red,
        WorkflowExecutionStatus::Running => BadgeColor::Blue,
        WorkflowExecutionStatus::Pending => BadgeColor::Zinc,
    };

    let status_text = match execution.status {
        WorkflowExecutionStatus::WaitingForInput => "Waiting for Input",
        WorkflowExecutionStatus::Complete => "Success",
        WorkflowExecutionStatus::Failed => "Failed",
        WorkflowExecutionStatus::Running => "Running",
        WorkflowExecutionStatus::Pending => "Pending",
    };

    let execution_id_for_nav = execution.id.clone();
    let execution_id_for_delete = execution.id.clone();
    let execution_id_for_dialog = execution.id.clone();
    let execution_name = execution.workflow_name.clone();
    let navigator_clone = navigator.clone();

    rsx! {
        div {
            ListItem {
                icon_name: "workflows".to_string(),
                icon_variant: Some("outline".to_string()),
                title: rsx! {
                    {execution.workflow_name.clone()}
                },
                subtitle: Some(rsx! {
                    div { class: "flex flex-col gap-1",
                        div { class: "text-sm text-gray-500 dark:text-gray-400",
                            "Executed: {execution.timestamp}"
                        }
                        div { class: "text-xs text-gray-500 dark:text-gray-400",
                            "{execution.task_count} tasks completed"
                        }
                    }
                }),
                right_content: Some(rsx! {
                    div { class: "flex items-center gap-2",
                        Badge {
                            color: badge_color,
                            {status_text}
                        }
                        IconButton {
                            variant: IconButtonVariant::Ghost,
                            size: IconButtonSize::Small,
                            onclick: Some(EventHandler::new(move |_| {
                                show_delete_dialog.set(true);
                            })),
                            icon: Some("trash".to_string()),
                            icon_variant: "outline".to_string(),
                            ""
                        }
                    }
                }),
                onclick: Some(EventHandler::new(move |_| {
                    if !show_delete_dialog() {
                        navigator_clone.push(Route::WorkflowDetailsPage { id: execution_id_for_nav.clone() });
                    }
                })),
            }

            ExecutionDeleteDialog {
                show: show_delete_dialog(),
                execution_id: execution_id_for_dialog.clone(),
                workflow_name: execution_name.clone(),
                on_confirm: move |_| {
                    show_delete_dialog.set(false);
                    let exec_id = execution_id_for_delete.clone();
                    let mutation = delete_mutation.clone();
                    spawn(async move {
                        let _reader = mutation.mutate_async(exec_id.clone()).await;
                    });
                },
                on_cancel: move |_| {
                    show_delete_dialog.set(false);
                },
            }
        }
    }
}
