use crate::components::{Button, ButtonSize, ButtonVariant, Slideout};
use crate::icons::Icon;
use dioxus::prelude::*;
use s_e_e_core::{TaskInfo, WorkflowExecution};

#[component]
fn TaskResumeButton(task: TaskInfo, execution_id: String) -> Element {
    let task_id = task.id.clone();

    rsx! {
        div {
            class: "mt-4 p-3 bg-amber-50 border border-amber-200 rounded",
            div { class: "flex items-center gap-2 mb-2",
                Icon {
                    name: "pause".to_string(),
                    class: Some("text-amber-600".to_string()),
                    size: Some("w-4 h-4".to_string()),
                    variant: Some("outline".to_string()),
                }
                span { class: "text-amber-800 font-medium", "Waiting for Input" }
            }
            p { class: "text-amber-700 text-sm mb-3",
                "This task is paused and waiting for user input."
            }
            button {
                class: "px-3 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded text-sm font-medium transition-colors inline-flex items-center gap-2",
                onclick: move |_| {
                    let execution_id_clone = execution_id.clone();
                    let task_id_clone = task_id.clone();

                    spawn(async move {
                        tracing::info!("Resume button clicked for task {}", task_id_clone);

                        match s_e_e_core::resume_task(&execution_id_clone, &task_id_clone).await {
                            Ok(_) => {
                                tracing::info!("Task resumed successfully");
                                // TODO: Refresh the page or update state in Phase 6
                            }
                            Err(e) => {
                                tracing::error!("Failed to resume task: {}", e);
                                // TODO: Show error message to user in Phase 6
                            }
                        }
                    });
                },
                Icon {
                    name: "play".to_string(),
                    class: Some("w-4 h-4".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
                "Resume Task"
            }
        }
    }
}

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
        match task.status.as_str() {
            "complete" => "bg-emerald-500/20 backdrop-blur-sm",
            "failed" => "bg-red-500/20 backdrop-blur-sm",
            "in-progress" => "bg-blue-500/20 backdrop-blur-sm",
            "pending" => "bg-zinc-500/20 backdrop-blur-sm",
            "waiting-for-input" => "bg-amber-500/20 backdrop-blur-sm",
            _ => "bg-zinc-500/20 backdrop-blur-sm",
        }
    } else {
        "bg-zinc-500/20 backdrop-blur-sm"
    };

    rsx! {
        Slideout {
            is_open,
            backdrop_class,
            on_close,
            title: if let Some(task) = current_task.as_ref() {
                task.name.clone()
            } else {
                "Task Details".to_string()
            },
            subtitle: current_task.as_ref().map(|task| format!("ID: {}", task.id)),
            show_close_button: Some(true),

            children: rsx! {
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
                                            class: "flex items-start justify-between text-sm py-2 border-b border-zinc-200 dark:border-zinc-700 last:border-b-0",
                                            div {
                                                class: "flex-1",
                                                div {
                                                    class: "text-zinc-500 dark:text-zinc-400 text-xs",
                                                    "{entry.timestamp}"
                                                }
                                                div {
                                                    class: "text-zinc-900 dark:text-zinc-100 mt-1",
                                                    "{entry.message}"
                                                }
                                            }
                                            div {
                                                class: format!("px-2 py-1 rounded text-xs font-medium ml-3 {}",
                                                    match entry.status {
                                                        s_e_e_core::AuditStatus::Success => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                                        s_e_e_core::AuditStatus::Failure => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                                    }
                                                ),
                                                "{entry.status}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Resume button for waiting tasks
                        if task.status == "waiting-for-input" {
                            if let Some(exec) = execution.as_ref() {
                                TaskResumeButton {
                                    task: task.clone(),
                                    execution_id: exec.id.clone()
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
            },

            footer: Some(rsx! {
                // Left side - Previous button
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

                // Center - Task status
                if let Some(task) = current_task.as_ref() {
                    div {
                        class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                            match task.status.as_str() {
                                "complete" => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                "failed" => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                "in-progress" => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                "pending" => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                                "waiting-for-input" => "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
                                _ => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                            }
                        ),
                        match task.status.as_str() {
                            "complete" => "Complete",
                            "failed" => "Failed",
                            "in-progress" => "In Progress",
                            "pending" => "Pending",
                            "waiting-for-input" => "Waiting for Input",
                            _ => "Unknown",
                        }
                    }
                }

                // Right side - Next button
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
            })
        }
    }
}
