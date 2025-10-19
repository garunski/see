use crate::{WorkflowResult, TaskInfo};
use dioxus::prelude::*;
use crate::components::{WorkflowProgress, StepNavigator};

#[component]
pub fn WorkflowInfoCard(
    result: ReadOnlySignal<WorkflowResult>,
    tasks: Vec<TaskInfo>,
    current_step: usize,
    on_next_step: EventHandler<()>,
    on_prev_step: EventHandler<()>,
    on_jump_to_step: EventHandler<usize>,
) -> Element {
    rsx! {
        div {
            class: "mb-8 bg-white dark:bg-zinc-900 shadow-xs ring-1 ring-zinc-950/5 dark:ring-white/10 rounded-2xl p-8 animate-fade-in",
            
            // Progress visualization
            if !tasks.is_empty() {
                div {
                    class: "mb-6",
                    WorkflowProgress {
                        tasks: tasks.clone(),
                        current_step: current_step,
                        on_step_click: move |step| on_jump_to_step.call(step),
                    }
                }
            }
            
            // Step navigator
            if !tasks.is_empty() {
                div {
                    class: "mb-6",
                    StepNavigator {
                        current_step: current_step,
                        total_steps: tasks.len(),
                        task_name: tasks.get(current_step).map(|t| t.name.clone()).unwrap_or_default(),
                        task_status: tasks.get(current_step).map(|t| t.status.clone()).unwrap_or_default(),
                        on_prev: move |_| on_prev_step.call(()),
                        on_next: move |_| on_next_step.call(()),
                    }
                }
            }
            
            div {
                class: "flex items-center justify-between mb-6",
                h2 {
                    class: "text-2xl/8 font-semibold text-zinc-950 sm:text-xl/8 dark:text-white",
                    "Workflow Results"
                }
                div {
                    class: "flex items-center space-x-2",
                    if result.read().success {
                        div {
                            class: "w-8 h-8 bg-emerald-500 rounded-full flex items-center justify-center text-white text-sm font-bold",
                            "✓"
                        }
                    } else {
                        div {
                            class: "w-8 h-8 bg-red-500 rounded-full flex items-center justify-center text-white text-sm font-bold",
                            "✕"
                        }
                    }
                }
            }

            // Stats grid
            div {
                class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                div {
                    class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                    div {
                        class: "text-zinc-500 dark:text-zinc-400 text-sm mb-2",
                        "Workflow Name"
                    }
                    div {
                        class: "text-zinc-950 dark:text-white text-lg font-semibold",
                        {result.read().workflow_name.clone()}
                    }
                }
                div {
                    class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                    div {
                        class: "text-zinc-500 dark:text-zinc-400 text-sm mb-2",
                        "Tasks"
                    }
                    div {
                        class: "text-zinc-950 dark:text-white text-lg font-semibold",
                        {result.read().task_count.to_string()}
                    }
                }
                div {
                    class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                    div {
                        class: "text-zinc-500 dark:text-zinc-400 text-sm mb-2",
                        "Status"
                    }
                    div {
                        class: "text-zinc-950 dark:text-white text-lg font-semibold",
                        {if result.read().success { "Success" } else { "Failed" }}
                    }
                }
            }
        }
    }
}
