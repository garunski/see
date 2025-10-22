use dioxus::prelude::*;
use see_core::WorkflowExecution;

#[component]
pub fn ExecutionOverview(execution: WorkflowExecution) -> Element {
    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{execution.workflow_name}" }
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
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div {
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Execution ID" }
                    div { class: "text-zinc-950 dark:text-white font-mono text-sm", "{execution.id}" }
                }
                div {
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Executed At" }
                    div { class: "text-zinc-950 dark:text-white", "{execution.timestamp}" }
                }
                div {
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Tasks" }
                    div { class: "text-zinc-950 dark:text-white", "{execution.tasks.len()} total" }
                }
            }
        }
    }
}
