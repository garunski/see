use dioxus::prelude::*;
use engine::TaskStatus;
use s_e_e_core::TaskInfo;

#[component]
pub fn WorkflowFlow(tasks: Vec<TaskInfo>, on_task_click: EventHandler<usize>) -> Element {
    if tasks.is_empty() {
        return rsx! {
            div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                div { class: "text-center text-zinc-500 dark:text-zinc-400",
                    "No tasks to display"
                }
            }
        };
    }

    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-6", "Workflow Flow" }

            div { class: "space-y-6 py-4",
                for (i, task) in tasks.iter().enumerate() {
                    div {
                        class: "relative",

                        // Top connection point indicator (not for first task)
                        if i > 0 {
                            div {
                                class: "absolute left-1/2 -top-2 transform -translate-x-1/2 w-4 h-4 rounded-full border-2 border-white dark:border-zinc-900 z-10",
                                style: format!("background-color: {}", match &task.status {
                                    TaskStatus::Complete => "#10b981",
                                    TaskStatus::Failed => "#ef4444",
                                    TaskStatus::InProgress => "#3b82f6",
                                    TaskStatus::Pending => "#6b7280",
                                    TaskStatus::WaitingForInput => "#f59e0b",
                                })
                            }
                        }

                        // Task card
                        div {
                            class: format!("relative bg-zinc-50 dark:bg-zinc-800 rounded-xl border p-6 shadow-sm cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors {}",
                                if matches!(task.status, TaskStatus::WaitingForInput) {
                                    "border-amber-300 dark:border-amber-700 animate-pulse"
                                } else {
                                    "border-zinc-200 dark:border-zinc-700"
                                }
                            ),
                            onclick: move |_| on_task_click.call(i),

                            // Bottom connection point indicator (except for last task)
                            if i < tasks.len() - 1 {
                                div {
                                    class: "absolute left-1/2 -bottom-2 transform -translate-x-1/2 w-4 h-4 rounded-full border-2 border-white dark:border-zinc-900 z-10",
                                    style: format!("background-color: {}", match &task.status {
                                        TaskStatus::Complete => "#10b981",
                                        TaskStatus::Failed => "#ef4444",
                                        TaskStatus::InProgress => "#3b82f6",
                                        TaskStatus::Pending => "#6b7280",
                                        TaskStatus::WaitingForInput => "#f59e0b",
                                    })
                                }
                            }

                            // Connecting line (except for last task)
                            if i < tasks.len() - 1 {
                                div {
                                    class: "absolute left-1/2 top-full transform -translate-x-1/2 w-0.5 h-6",
                                    style: format!("background-color: {}", match &task.status {
                                        TaskStatus::Complete => "#10b981",
                                        TaskStatus::Failed => "#ef4444",
                                        TaskStatus::InProgress => "#3b82f6",
                                        TaskStatus::Pending => "#6b7280",
                                        TaskStatus::WaitingForInput => "#f59e0b",
                                    })
                                }
                            }

                            div { class: "flex items-center justify-between",
                                h4 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{task.name}" }
                                div {
                                    class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                        match &task.status {
                                            TaskStatus::Complete => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                            TaskStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                            TaskStatus::InProgress => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                            TaskStatus::Pending => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                                            TaskStatus::WaitingForInput => "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
                                        }
                                    ),
                                    match &task.status {
                                        TaskStatus::Complete => "Complete",
                                        TaskStatus::Failed => "Failed",
                                        TaskStatus::InProgress => "In Progress",
                                        TaskStatus::Pending => "Pending",
                                        TaskStatus::WaitingForInput => "Waiting for Input",
                                    }
                                }
                            }

                            div { class: "mt-2 text-sm text-zinc-500 dark:text-zinc-400",
                                "Task ID: {task.id}"
                            }
                        }
                    }
                }
            }
        }
    }
}
