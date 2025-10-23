mod components;
mod hooks;

use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::state::AppStateProvider;
use components::{ErrorBanner, HistoryItem, LoadingSkeleton, RunningWorkflowItem};
use dioxus::prelude::*;
use hooks::{use_deletion_handlers, use_history_data};

#[component]
pub fn HistoryPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();

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

    // Get data from state
    let workflow_history = use_memo(move || state_provider.history.read().workflow_history.clone());
    let running_workflows =
        use_memo(move || state_provider.history.read().running_workflows.clone());

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Workflow History" }
                    p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "View and manage your previous workflow executions" }
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Medium,
                    disabled: Some(is_loading()),
                    loading: Some(is_loading()),
                    onclick: move |_| refresh_data_button(),
                    class: "flex items-center gap-2".to_string(),
                    svg {
                        class: "w-4 h-4",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                    }
                    span { "Refresh" }
                }
            }

            if let Some(err) = error() {
                ErrorBanner {
                    error: err,
                    on_retry: move |_| refresh_data_error()
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
                            for workflow in running_workflows().iter() {
                                RunningWorkflowItem {
                                    workflow: workflow.clone(),
                                    on_delete_workflow: delete_running_workflow.clone(),
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-4",
                    div { class: "flex items-center justify-between",
                        h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Completed Workflows" }
                    }
                    if workflow_history().is_empty() {
                        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-16 text-center shadow-sm",
                            div { class: "text-6xl mb-6", "📋" }
                            h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-3", "No completed workflows yet" }
                            p { class: "text-zinc-500 dark:text-zinc-400", "Execute your first workflow to see it appear here" }
                        }
                    } else {
                        div { class: "grid gap-6",
                            for execution in workflow_history().iter() {
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
