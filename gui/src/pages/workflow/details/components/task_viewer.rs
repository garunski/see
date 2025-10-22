use super::TaskDetails;
use crate::pages::workflow::upload::components::{StepNavigator, WorkflowProgress};
use dioxus::prelude::*;
use see_core::{TaskInfo, TaskStatus};

#[derive(Props, PartialEq, Clone)]
pub struct TaskViewerProps {
    pub tasks: Vec<TaskInfo>,
    pub current_step: usize,
    pub on_step_click: EventHandler<usize>,
    pub on_prev: EventHandler<()>,
    pub on_next: EventHandler<()>,
}

#[component]
pub fn TaskViewer(props: TaskViewerProps) -> Element {
    let current_task = props.tasks.get(props.current_step);
    let total_tasks = props.tasks.len();

    rsx! {
        div {
            class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            h3 {
                class: "text-base font-semibold text-zinc-950 dark:text-white mb-4",
                "Current Task"
            }

            WorkflowProgress {
                tasks: props.tasks.clone(),
                current_step: props.current_step,
                on_step_click: props.on_step_click
            }

            if let Some(task) = current_task {
                TaskDetails {
                    task: task.clone()
                }
            }

            div {
                class: "mt-4",
                StepNavigator {
                    current_step: props.current_step,
                    total_steps: total_tasks,
                    task_name: current_task.map(|t| t.name.clone()).unwrap_or_default(),
                    task_status: current_task.map(|t| t.status.clone()).unwrap_or(TaskStatus::Pending),
                    on_prev: props.on_prev,
                    on_next: props.on_next
                }
            }
        }
    }
}
