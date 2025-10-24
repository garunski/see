use dioxus::prelude::*;
use s_e_e_core::TaskInfo;

#[component]
pub fn TaskLogs(
    current_task: Option<TaskInfo>,
    per_task_logs: std::collections::HashMap<String, Vec<String>>,
) -> Element {
    let logs = current_task
        .and_then(|task| per_task_logs.get(&task.id))
        .filter(|logs: &&Vec<String>| !logs.is_empty());

    if let Some(logs) = logs {
        rsx! {
            div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Current Task Logs" }
                div { class: "space-y-2 max-h-96 overflow-y-auto",
                    for log in logs.iter() {
                        div { class: "text-sm text-zinc-700 dark:text-zinc-300 font-mono bg-zinc-100 dark:bg-zinc-800 p-2 rounded", "{log}" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
