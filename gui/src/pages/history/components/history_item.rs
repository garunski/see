use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::WorkflowExecutionSummary;

#[component]
pub fn HistoryItem(
    execution: WorkflowExecutionSummary,
    on_delete_execution: EventHandler<String>,
) -> Element {
    let execution_id_for_delete = execution.id.clone();
    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            div { class: "flex items-center justify-between p-6",
                Link {
                    to: Route::WorkflowDetailsPage { id: execution.id.clone() },
                    class: "flex-1 min-w-0 cursor-pointer",
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
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Small,
                    onclick: move |_| {
                        on_delete_execution.call(execution_id_for_delete.clone());
                    },
                    class: "ml-4 p-2 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20".to_string(),
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
