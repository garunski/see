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

    let is_clickable = task.has_execution_data;
    let cursor_class = if is_clickable {
        "cursor-pointer"
    } else {
        "cursor-not-allowed"
    };
    let hover_class = if is_clickable {
        "hover:bg-zinc-100 dark:hover:bg-zinc-700"
    } else {
        ""
    };

    rsx! {
        div { class: "min-w-[200px]",
            div {
                class: "{cursor_class} {hover_class} transition-colors",
                onclick: move |_| {
                    if is_clickable {
                        navigator.push(Route::WorkflowDetailsTaskDetailsPage {
                            execution_id: execution_id.clone(),
                            task_id: task.id.clone()
                        });
                    }
                },
                div { class: "flex rounded-md shadow-sm dark:shadow-none",
                    div {
                        class: "flex w-16 shrink-0 items-center justify-center rounded-l-md {task.function_color} text-sm font-medium text-white relative",
                        Icon {
                            name: task.function_icon.to_string(),
                            size: Some("w-6 h-6".to_string()),
                            variant: Some("outline".to_string()),
                            class: Some("".to_string()),
                        }

                        div {
                            class: "{task.status_color} absolute -top-1 -right-1 w-6 h-6 rounded-full flex items-center justify-center border-2 border-white dark:border-gray-900",
                            Icon {
                                name: task.status_icon.to_string(),
                                size: Some("w-3 h-3".to_string()),
                                variant: Some("outline".to_string()),
                                class: Some("".to_string()),
                            }
                        }
                    }
                    div { class: "flex flex-1 items-center justify-between truncate rounded-r-md border-b border-r border-t border-gray-200 bg-white dark:border-white/10 dark:bg-gray-800/50",
                        div { class: "flex-1 truncate px-4 py-2 text-sm",
                            div { class: "font-medium text-gray-900 dark:text-white truncate", "{task.name}" }
                            p { class: "text-gray-500 dark:text-gray-400 text-xs truncate", "{task.function_name}" }
                        }
                    }
                }
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
