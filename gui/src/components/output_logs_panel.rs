use dioxus::prelude::*;
use see_core::TaskInfo;
use std::collections::HashMap;

#[component]
pub fn OutputLogsPanel(
    per_task_logs: HashMap<String, Vec<String>>,
    tasks: Vec<TaskInfo>,
    current_step: usize,
    show_logs: bool,
    on_toggle: EventHandler<()>,
    on_copy: EventHandler<String>,
) -> Element {
    let current_task_logs = if let Some(current_task) = tasks.get(current_step) {
        per_task_logs
            .get(&current_task.id)
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let current_task = tasks.get(current_step);
    let task_name = current_task.map(|t| t.name.clone()).unwrap_or_default();
    let task_status = current_task.map(|t| t.status.clone()).unwrap_or_default();

    let display_text = if !tasks.is_empty() {
        format!("Step {} Output: {}", current_step + 1, task_name)
    } else {
        "Execution Output".to_string()
    };
    let line_count_text = if !current_task_logs.is_empty() {
        format!("({} lines)", current_task_logs.len())
    } else {
        "(No logs for this step)".to_string()
    };

    rsx! {
        if !current_task_logs.is_empty() || !tasks.is_empty() {
            div { class: "bg-white dark:bg-zinc-900 shadow-xs ring-1 ring-zinc-950/5 dark:ring-white/10 rounded-2xl overflow-hidden",
                button { class: "w-full px-6 py-4 text-left flex items-center justify-between data-hover:bg-zinc-950/5 dark:data-hover:bg-white/5 transition-colors duration-200", onclick: move |_| on_toggle.call(()),
                    div { class: "flex items-center space-x-3",
                        div { class: "w-8 h-8 bg-blue-500/20 rounded-lg flex items-center justify-center", "📋" }
                        div {
                            span { class: "font-semibold text-zinc-950 dark:text-white text-lg", {display_text} }
                            span { class: "text-sm text-zinc-500 dark:text-zinc-400 ml-2", {line_count_text} }
                        }
                        if !tasks.is_empty() {
                            span { class: match task_status.as_str() {
                                    "complete" => "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800 dark:bg-emerald-900/20 dark:text-emerald-400",
                                    "failed" => "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400",
                                    "in-progress" => "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400",
                                    _ => "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400",
                                }, {task_status.clone()} }
                        }
                    }
                    div { class: "transform transition-transform duration-200", class: if show_logs { "rotate-180" } else { "" }, "▼" }
                }
                if show_logs {
                    div { class: "border-t border-zinc-950/5 dark:border-white/5",
                        div { class: "bg-zinc-50 dark:bg-zinc-950 text-zinc-950 dark:text-zinc-100 p-6 font-mono text-sm max-h-80 overflow-y-auto",
                            if !current_task_logs.is_empty() { {current_task_logs.join("\n")} } else { "No logs available for this step." }
                        }
                        div { class: "px-6 py-4 bg-zinc-50 dark:bg-zinc-800 flex justify-end",
                            button { class: "px-4 py-2 bg-blue-500/20 hover:bg-blue-500/30 text-blue-600 dark:text-blue-400 rounded-xl text-sm font-medium transition-colors duration-200 border border-blue-500/30", onclick: move |_| { let logs_text = current_task_logs.join("\n"); on_copy.call(logs_text); }, "Copy Output" }
                        }
                    }
                }
            }
        }
    }
}
