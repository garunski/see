use crate::components::SectionCard;
use dioxus::prelude::*;
use s_e_e_core::TaskExecution;

#[component]
pub fn TaskDetailsInfoTab(task: TaskExecution) -> Element {
    rsx! {
        SectionCard {
            title: Some("Task Information".to_string()),
            children: rsx! {
                div { class: "space-y-3",
                    div { class: "flex justify-between",
                        span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Name:" }
                        span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{task.name}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Status:" }
                        span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{task.status:?}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "ID:" }
                        span { class: "text-sm text-zinc-900 dark:text-zinc-100 font-mono", "{task.id}" }
                    }
                    if let Some(error) = task.error.as_ref() {
                        div { class: "flex justify-between",
                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Error:" }
                            span { class: "text-sm text-red-600 dark:text-red-400", "{error}" }
                        }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Created At:" }
                        span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{task.created_at}" }
                    }
                    if let Some(completed_at) = task.completed_at.as_ref() {
                        div { class: "flex justify-between",
                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Completed At:" }
                            span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{completed_at}" }
                        }
                    }
                }
            },
            padding: None,
        }
    }
}
