use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::icons::Icon;
use dioxus::prelude::*;
use see_core::{TaskInfo, WorkflowExecution};

#[allow(dead_code)]
#[component]
pub fn TaskDetailsPanel(
    is_open: bool,
    current_task: Option<TaskInfo>,
    current_task_index: usize,
    total_tasks: usize,
    execution: Option<WorkflowExecution>,
    on_close: EventHandler<()>,
    on_previous: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! { div {} };
    }

    let can_go_previous = current_task_index > 0;
    let can_go_next = current_task_index < total_tasks.saturating_sub(1);

    // Pre-calculate task audit to avoid issues in rsx!
    let task_audit = if let (Some(exec), Some(task)) = (execution.as_ref(), current_task.as_ref()) {
        exec.audit_trail
            .iter()
            .filter(|entry| entry.task_id == task.id)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Determine backdrop color based on task status
    let backdrop_class = if let Some(task) = current_task.as_ref() {
        match task.status {
            see_core::TaskStatus::Complete => "bg-emerald-500/20 backdrop-blur-sm",
            see_core::TaskStatus::Failed => "bg-red-500/20 backdrop-blur-sm",
            see_core::TaskStatus::InProgress => "bg-blue-500/20 backdrop-blur-sm",
            see_core::TaskStatus::Pending => "bg-zinc-500/20 backdrop-blur-sm",
        }
    } else {
        "bg-zinc-500/20 backdrop-blur-sm"
    };

    rsx! {
        // Backdrop
        div {
            class: format!("fixed inset-0 z-50 {}", backdrop_class),
            onclick: move |_| on_close.call(()),

            // Panel
            div {
                class: "fixed inset-y-0 right-0 z-50 w-3/4 transform transition-transform duration-500 ease-in-out sm:duration-700",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "flex h-full flex-col divide-y divide-gray-200 bg-white shadow-xl dark:divide-white/10 dark:bg-gray-800 dark:after:absolute dark:after:inset-y-0 dark:after:left-0 dark:after:w-px dark:after:bg-white/10",

                        // Header with title and close button
                        div {
                            class: "flex items-center justify-between px-4 py-4 sm:px-6",
                            div {
                                h2 {
                                    class: "text-lg font-semibold text-gray-900 dark:text-white",
                                    if let Some(task) = current_task.as_ref() {
                                        "{task.name}"
                                    } else {
                                        "Task Details"
                                    }
                                }
                                if let Some(task) = current_task.as_ref() {
                                    div {
                                        class: "text-sm text-gray-500 dark:text-gray-400 mt-1",
                                        "ID: {task.id}"
                                    }
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Small,
                                onclick: Some(on_close),
                                class: Some("text-gray-400 hover:text-gray-500 dark:hover:text-white".to_string()),
                                Icon {
                                    name: "x".to_string(),
                                    class: Some("w-6 h-6".to_string()),
                                    size: None,
                                    variant: Some("outline".to_string()),
                                }
                            }
                        }

                    // Content
                    div {
                        class: "flex-1 overflow-y-auto py-6",
                        div {
                            class: "px-4 sm:px-6",

                            if let Some(task) = current_task.as_ref() {
                                div {
                                    class: "space-y-6",


                                    // Task logs
                                    if let Some(exec) = execution.as_ref() {
                                        if let Some(logs) = exec.per_task_logs.get(&task.id) {
                                            if !logs.is_empty() {
                                                div {
                                                    class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                                                    h4 {
                                                        class: "text-base font-semibold text-zinc-950 dark:text-white mb-4",
                                                        "Task Logs"
                                                    }
                                                    div {
                                                        class: "space-y-2",
                                                        for log in logs.iter() {
                                                            div {
                                                                class: "text-sm text-zinc-700 dark:text-zinc-300 font-mono bg-white dark:bg-zinc-900 p-3 rounded border break-words overflow-wrap-anywhere whitespace-pre-wrap",
                                                                "{log}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Audit trail for this task
                                    if !task_audit.is_empty() {
                                        div {
                                            class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                                            h4 {
                                                class: "text-base font-semibold text-zinc-950 dark:text-white mb-4",
                                                "Audit Trail"
                                            }
                                            div {
                                                class: "space-y-3",
                                                for entry in task_audit.iter() {
                                                    div {
                                                        class: "flex items-center justify-between text-sm",
                                                        div {
                                                            class: "text-zinc-500 dark:text-zinc-400",
                                                            "{entry.timestamp}"
                                                        }
                                                        div {
                                                            class: format!("px-2 py-1 rounded text-xs font-medium {}",
                                                                match entry.status {
                                                                    see_core::AuditStatus::Success => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                                                    see_core::AuditStatus::Failure => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                                                }
                                                            ),
                                                            "{entry.status}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    class: "text-center text-zinc-500 dark:text-zinc-400",
                                    "No task selected"
                                }
                            }
                        }
                    }

                        // Footer with navigation
                        div {
                            class: "flex shrink-0 items-center justify-between px-4 py-4",

                            // Previous button
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Small,
                                disabled: Some(!can_go_previous),
                                onclick: Some(on_previous),
                                class: Some("text-gray-400 hover:text-gray-500 dark:hover:text-white".to_string()),
                                Icon {
                                    name: "chevron_left".to_string(),
                                    class: Some("w-5 h-5".to_string()),
                                    size: None,
                                    variant: Some("outline".to_string()),
                                }
                            }

                            // Task status in center
                            if let Some(task) = current_task.as_ref() {
                                div {
                                    class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                        match task.status {
                                            see_core::TaskStatus::Complete => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                            see_core::TaskStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                            see_core::TaskStatus::InProgress => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                            see_core::TaskStatus::Pending => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                                        }
                                    ),
                                    match task.status {
                                        see_core::TaskStatus::Complete => "Complete",
                                        see_core::TaskStatus::Failed => "Failed",
                                        see_core::TaskStatus::InProgress => "In Progress",
                                        see_core::TaskStatus::Pending => "Pending",
                                    }
                                }
                            }

                            // Next button
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Small,
                                disabled: Some(!can_go_next),
                                onclick: Some(on_next),
                                class: Some("text-gray-400 hover:text-gray-500 dark:hover:text-white".to_string()),
                                Icon {
                                    name: "chevron_right".to_string(),
                                    class: Some("w-5 h-5".to_string()),
                                    size: None,
                                    variant: Some("outline".to_string()),
                                }
                            }
                        }
                }
            }
        }
    }
}
