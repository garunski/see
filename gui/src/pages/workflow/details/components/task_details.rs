use super::super::utils::{format_task_status, get_task_status_badge_variant};
use crate::components::ui::Badge;
use dioxus::prelude::*;
use see_core::TaskInfo;

#[derive(Props, PartialEq, Clone)]
pub struct TaskDetailsProps {
    pub task: TaskInfo,
}

#[component]
pub fn TaskDetails(props: TaskDetailsProps) -> Element {
    let task = &props.task;
    let status_variant = get_task_status_badge_variant(&task.status);
    let status_text = format_task_status(&task.status);

    rsx! {
        div {
            class: "mt-6 p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg",
            div {
                class: "flex items-center justify-between mb-3",
                h4 {
                    class: "text-base font-semibold text-zinc-950 dark:text-white",
                    {task.name.clone()}
                }
                Badge {
                    variant: status_variant,
                    {status_text}
                }
            }
            div {
                class: "text-sm text-zinc-500 dark:text-zinc-400",
                "Task ID: {task.id}"
            }
        }
    }
}
