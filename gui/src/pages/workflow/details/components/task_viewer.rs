use super::{StepNavigator, WorkflowProgress};
use dioxus::prelude::*;
use see_core::{TaskInfo, TaskStatus};

#[component]
pub fn TaskViewer(
    tasks: Vec<TaskInfo>,
    current_step: usize,
    on_step_click: EventHandler<usize>,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    let total_tasks = tasks.len();
    let current_task = tasks.get(current_step);

    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Current Task" }

            WorkflowProgress {
                tasks: tasks.clone(),
                current_step,
                on_step_click
            }

            if let Some(task) = current_task {
                div { class: "mt-6 p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg",
                    div { class: "flex items-center justify-between mb-3",
                        h4 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{task.name}" }
                        div {
                            class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                match task.status {
                                    TaskStatus::Complete => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                    TaskStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                    TaskStatus::InProgress => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                    TaskStatus::Pending => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                                }
                            ),
                            match task.status {
                                TaskStatus::Complete => "Complete",
                                TaskStatus::Failed => "Failed",
                                TaskStatus::InProgress => "In Progress",
                                TaskStatus::Pending => "Pending",
                            }
                        }
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Task ID: {task.id}"
                    }
                }
            }

            div { class: "mt-4",
                StepNavigator {
                    current_step,
                    total_steps: total_tasks,
                    task_name: current_task.map(|t| t.name.clone()).unwrap_or_default(),
                    task_status: current_task.map(|t| t.status.clone()).unwrap_or(TaskStatus::Pending),
                    on_prev,
                    on_next
                }
            }
        }
    }
}
