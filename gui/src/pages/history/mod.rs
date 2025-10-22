use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::{AuditStore, WorkflowExecutionSummary};
use std::sync::Arc;

#[component]
fn HistoryItem(
    execution: WorkflowExecutionSummary,
    on_delete_execution: EventHandler<String>,
) -> Element {
    let execution_id_for_delete = execution.id.clone();
    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            Link {
                to: Route::WorkflowDetailsPage { id: execution.id.clone() },
                class: "block p-6",
                div { class: "flex items-center justify-between",
                    div { class: "flex-1 min-w-0",
                        div { class: "flex items-center gap-4 mb-3",
                            h4 { class: "text-base font-semibold text-zinc-900 dark:text-white truncate", "{execution.workflow_name}" }
                            div {
                                class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                    if execution.success {
                                        "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
                                    } else {
                                        "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
                                    }
                                ),
                                if execution.success { "Success" } else { "Failed" }
                            }
                        }
                        div { class: "text-sm text-zinc-500 dark:text-zinc-400 mb-2",
                            "Executed: {execution.timestamp}"
                        }
                        div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                            "{execution.task_count} tasks completed"
                        }
                    }
                    button {
                        class: "ml-4 p-2 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 transition-colors rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_delete_execution.call(execution_id_for_delete.clone());
                        },
                        // TrashIcon SVG
                        svg {
                            class: "w-5 h-5",
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            path { d: "M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn HistoryPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let store = use_context::<Option<Arc<see_core::RedbStore>>>();

    let workflow_history = use_memo(move || state_provider.history.read().workflow_history.clone());

    let store_clone = store.clone();
    let delete_execution = {
        let state_provider = state_provider.clone();
        move |id: String| {
            let store_clone = store_clone.clone();
            let mut history_state = state_provider.history;
            let _ui_state = state_provider.ui;
            // Status updates removed
            spawn(async move {
                if let Some(s) = store_clone {
                    match s.delete_workflow_execution(&id).await {
                        Ok(_) => {
                            history_state.write().delete_execution(&id);
                            // Status updates removed
                        }
                        Err(_e) => {
                            // Status updates removed
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "space-y-8",
            // Header
            div {
                h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Workflow History" }
                p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "View and manage your previous workflow executions" }
            }

            // History List
            if workflow_history().is_empty() {
                div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-16 text-center shadow-sm",
                    div { class: "text-6xl mb-6", "📋" }
                    h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-3", "No workflow executions yet" }
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
