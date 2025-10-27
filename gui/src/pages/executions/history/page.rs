use crate::components::{Alert, AlertType, Button, ButtonSize, ButtonVariant, PageHeader};
use crate::hooks::{use_running_workflows, use_workflow_history};
use crate::icons::Icon;
use crate::state::AppStateProvider;
use dioxus::prelude::*;

use super::components::{HistoryItem, LoadingSkeleton, RunningWorkflowItem};
use super::hooks::{use_deletion_handlers, use_history_data};

#[component]
pub fn HistoryPage() -> Element {
    tracing::debug!("rendering history page");
    let _state_provider = use_context::<AppStateProvider>();

    // Use custom hooks for data management
    let (is_loading, error, refresh_data) = use_history_data();
    let (delete_execution, delete_running_workflow) = use_deletion_handlers();

    // Initial data load
    let refresh_data_clone = refresh_data.clone();
    use_effect(move || {
        refresh_data_clone();
    });

    // Clone for button and error banner
    let refresh_data_button = refresh_data.clone();
    let refresh_data_error = refresh_data.clone();

    // Get data from state using hooks
    let workflow_history = use_workflow_history();
    let running_workflows = use_running_workflows();

    // Separate workflows into categories
    let workflow_categories = use_memo(move || {
        use s_e_e_core::WorkflowExecutionStatus;
        let history = workflow_history();
        let (waiting, completed) = history.into_iter().partition::<Vec<_>, _>(|exec| {
            matches!(
                exec.status,
                WorkflowExecutionStatus::WaitingForInput | WorkflowExecutionStatus::Running
            )
        });
        (waiting, completed)
    });
    let waiting_workflows = use_memo(move || workflow_categories().0);
    let completed_workflows = use_memo(move || workflow_categories().1);

    // Log what we're displaying
    use_effect(move || {
        let running = running_workflows();
        let history = workflow_history();
        tracing::debug!(
            running_count = running.len(),
            running_ids = ?running.iter().map(|w| &w.id).collect::<Vec<_>>(),
            completed_count = history.len(),
            completed_ids = ?history.iter().map(|h| &h.id).collect::<Vec<_>>(),
            "history page rendering"
        );
    });

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Executions".to_string(),
                description: "View and manage your workflow executions".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Medium,
                        disabled: Some(is_loading()),
                        loading: Some(is_loading()),
                        onclick: move |_| refresh_data_button(),
                        class: "flex items-center gap-2".to_string(),
                        Icon {
                            name: "history".to_string(),
                            class: Some("w-4 h-4".to_string()),
                            size: None,
                            variant: Some("outline".to_string()),
                        }
                        span { "Refresh" }
                    }
                }),
            }

            if let Some(err) = error() {
                Alert {
                    alert_type: AlertType::Error,
                    title: Some("Failed to load history".to_string()),
                    message: err,
                    dismissible: None,
                    on_dismiss: None,
                    actions: Some(rsx! {
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Small,
                            onclick: move |_| refresh_data_error(),
                            class: "px-3 py-1 text-sm font-medium text-red-800 dark:text-red-200 bg-red-100 dark:bg-red-900/30 hover:bg-red-200 dark:hover:bg-red-900/50".to_string(),
                            "Retry"
                        }
                    }),
                }
            }

            if is_loading() && workflow_history().is_empty() && running_workflows().is_empty() {
                LoadingSkeleton {}
            } else {
                if !running_workflows().is_empty() {
                    div { class: "space-y-4",
                        div { class: "flex items-center justify-between",
                            h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Running Workflows" }
                            div { class: "flex items-center gap-2",
                                div { class: "w-2 h-2 bg-blue-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-zinc-500 dark:text-zinc-400", "Live updates" }
                            }
                        }
                        div { class: "grid gap-6",
                            for (index, workflow) in running_workflows().iter().enumerate() {
                                {
                                    tracing::debug!(
                                        running_index = index + 1,
                                        execution_id = %workflow.id,
                                        workflow_name = %workflow.workflow_name,
                                        status = ?workflow.status,
                                        "Rendering running workflow"
                                    );
                                }
                                RunningWorkflowItem {
                                    workflow: workflow.clone(),
                                    on_delete_workflow: delete_running_workflow.clone(),
                                }
                            }
                        }
                    }
                }

                if !waiting_workflows().is_empty() {
                    div { class: "space-y-4",
                        div { class: "flex items-center justify-between",
                            h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Waiting for Input" }
                            div { class: "flex items-center gap-2",
                                Icon {
                                    name: "pause".to_string(),
                                    class: Some("w-4 h-4 text-amber-600 dark:text-amber-400".to_string()),
                                    size: None,
                                    variant: Some("outline".to_string()),
                                }
                                span { class: "text-sm text-amber-600 dark:text-amber-400", "Action required" }
                            }
                        }
                        div { class: "grid gap-6",
                            for execution in waiting_workflows().iter() {
                                HistoryItem {
                                    execution: execution.clone(),
                                    on_delete_execution: delete_execution.clone(),
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-4",
                    div { class: "flex items-center justify-between",
                        h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Completed Workflows" }
                    }
                    if completed_workflows().is_empty() {
                        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-16 text-center shadow-sm",
                            div { class: "flex justify-center mb-6",
                                Icon {
                                    name: "history".to_string(),
                                    class: Some("w-16 h-16 text-zinc-400 dark:text-zinc-500".to_string()),
                                    size: None,
                                    variant: Some("outline".to_string()),
                                }
                            }
                            h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-3", "No completed workflows yet" }
                            p { class: "text-zinc-500 dark:text-zinc-400", "Execute your first workflow to see it appear here" }
                        }
                    } else {
                        div { class: "grid gap-6",
                            for (index, execution) in completed_workflows().iter().enumerate() {
                                {
                                    tracing::trace!(
                                        completed_index = index + 1,
                                        execution_id = %execution.id,
                                        workflow_name = %execution.workflow_name,
                                        status = %execution.status,
                                        "Rendering completed workflow"
                                    );
                                }
                                HistoryItem {
                                    execution: execution.clone(),
                                    on_delete_execution: delete_execution.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
