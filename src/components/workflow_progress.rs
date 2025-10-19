use crate::TaskInfo;
use dioxus::prelude::*;

#[component]
pub fn WorkflowProgress(
    tasks: Vec<TaskInfo>,
    current_step: usize,
    on_step_click: EventHandler<usize>,
) -> Element {
    let total_steps = tasks.len();

    // Pre-calculate step data
    let step_data: Vec<_> = tasks.iter().enumerate().map(|(step_idx, task)| {
        let is_last = step_idx == total_steps.saturating_sub(1);
        let step_number = step_idx + 1;
        // Determine step status
        let status = if step_idx < current_step {
            if task.status == "failed" { "failed" } else { "complete" }
        } else if step_idx == current_step {
            "current"
        } else {
            "upcoming"
        };
        // Pre-calculate classes
        let button_class = match status {
            "complete" if task.status != "failed" => "relative flex size-8 items-center justify-center rounded-full bg-emerald-600 hover:bg-emerald-900 dark:bg-emerald-500 dark:hover:bg-emerald-400 text-white font-bold",
            "failed" => "relative flex size-8 items-center justify-center rounded-full bg-red-600 hover:bg-red-900 dark:bg-red-500 dark:hover:bg-red-400 text-white font-bold",
            "current" => "relative flex size-8 items-center justify-center rounded-full border-2 border-blue-600 bg-white dark:border-blue-500 dark:bg-gray-900",
            _ => "group relative flex size-8 items-center justify-center rounded-full border-2 border-gray-300 bg-white hover:border-gray-400 dark:border-white/15 dark:bg-gray-900 dark:hover:border-white/25"
        };
        let line_class = match status {
            "complete" if task.status != "failed" => "h-0.5 w-full bg-emerald-600 dark:bg-emerald-500",
            "failed" => "h-0.5 w-full bg-red-600 dark:bg-red-500",
            _ => "h-0.5 w-full bg-gray-200 dark:bg-white/15"
        };
        let number_class = match status {
            "current" => "text-blue-600 dark:text-blue-500",
            _ => "text-gray-500 dark:text-gray-400"
        };
        (step_idx, task, is_last, step_number, status, button_class, line_class, number_class)
    }).collect();

    rsx! {
        nav {
            "aria-label": "Progress",
            ol {
                role: "list",
                class: "flex items-center",
                for (step_idx, task, is_last, step_number, status, button_class, line_class, number_class) in step_data {

                    li {
                        class: if is_last { "relative" } else { "relative pr-8 sm:pr-20" },

                        // Connecting line (except for last step)
                        if !is_last {
                            div {
                                "aria-hidden": "true",
                                class: "absolute inset-0 flex items-center",
                                div {
                                    class: line_class
                                }
                            }
                        }

                        // Step button
                        button {
                            class: button_class,
                            onclick: move |_| on_step_click.call(step_idx),

                            if status == "complete" && task.status != "failed" {
                                // Checkmark for completed steps
                                span {
                                    "aria-hidden": "true",
                                    class: "size-5",
                                    "✓"
                                }
                            } else if status == "failed" {
                                // X for failed steps
                                span {
                                    "aria-hidden": "true",
                                    class: "size-5",
                                    "✕"
                                }
                            } else if status == "current" {
                                // Inner dot for current step
                                span {
                                    "aria-hidden": "true",
                                    class: "size-2.5 rounded-full bg-blue-600 dark:bg-blue-500"
                                }
                            } else {
                                // Empty circle for upcoming steps
                                span {
                                    "aria-hidden": "true",
                                    class: "size-2.5 rounded-full bg-transparent group-hover:bg-gray-300 dark:group-hover:bg-white/15"
                                }
                            }

                            // Step number (always visible)
                            if status != "complete" || task.status == "failed" {
                                span {
                                    class: format!("absolute inset-0 flex items-center justify-center text-xs font-bold {}", number_class),
                                    {step_number.to_string()}
                                }
                            }

                            // Screen reader label
                            span {
                                class: "sr-only",
                                "Step {step_number}: {task.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
