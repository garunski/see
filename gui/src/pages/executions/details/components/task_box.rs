use crate::icons::Icon;
use crate::layout::router::Route;
use crate::pages::executions::details::components::task_preprocessing::RenderableTask;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[derive(Props, PartialEq, Clone)]
pub struct TaskBoxProps {
    pub task: RenderableTask,
    pub execution_id: String,
}

#[component]
pub fn TaskBox(props: TaskBoxProps) -> Element {
    let TaskBoxProps { task, execution_id } = props;
    let navigator = use_navigator();

    rsx! {
        div { class: "min-w-[120px]",
            div {
                class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-6 shadow-sm cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors relative",
                onclick: move |_| {
                    navigator.push(Route::WorkflowDetailsTaskDetailsPage {
                        execution_id: execution_id.clone(),
                        task_id: task.id.clone()
                    });
                },
                // Status icon - absolute positioned in top right
                div {
                    class: "{task.status_color} absolute top-2 right-2 w-8 h-8 rounded-full flex items-center justify-center",
                    Icon {
                        name: task.status_icon.to_string(),
                        size: Some("w-4 h-4".to_string()),
                        variant: Some("outline".to_string()),
                        class: Some("".to_string()),
                    }
                }
                div { class: "flex items-start justify-between mb-2",
                    h4 { class: "text-base font-semibold text-zinc-950 dark:text-white flex-1 pr-10 truncate", "{task.name}" }
                }
                div { class: "text-sm text-zinc-500 dark:text-zinc-400 truncate", "ID: {task.id}" }
            }

            if !task.children.is_empty() {
                div { class: "grid gap-4 mt-4 overflow-x-auto", style: format!("grid-template-columns: repeat({}, minmax(120px, 1fr)); min-width: {}px", task.children.len(), task.children.len() * 140),
                    for child in task.children.iter() {
                        TaskBox {
                            task: child.clone(),
                            execution_id: execution_id.clone(),
                        }
                    }
                }
            }
        }
    }
}
