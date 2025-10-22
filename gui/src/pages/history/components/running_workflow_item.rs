use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::WorkflowMetadata;

#[component]
pub fn RunningWorkflowItem(
    workflow: WorkflowMetadata,
    on_delete_workflow: EventHandler<String>,
) -> Element {
    let workflow_id_for_delete = workflow.id.clone();
    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            div { class: "flex items-center justify-between p-6",
                Link {
                    to: Route::WorkflowDetailsPage { id: workflow.id.clone() },
                    class: "flex-1 min-w-0 cursor-pointer",
                    div { class: "flex items-center gap-4 mb-3",
                        h4 { class: "text-base font-semibold text-zinc-900 dark:text-white truncate", "{workflow.workflow_name}" }
                        div {
                            class: "px-3 py-1 text-sm rounded-full font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                            "In Progress"
                        }
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400 mb-2",
                        "Started: {workflow.start_timestamp}"
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "{workflow.task_ids.len()} tasks"
                    }
                }
                div { class: "ml-4 flex items-center gap-2",
                    svg {
                        class: "w-5 h-5 text-blue-600 dark:text-blue-400 animate-spin",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                    }
                    button {
                        class: "p-2 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 transition-colors rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20",
                        onclick: move |_| {
                            on_delete_workflow.call(workflow_id_for_delete.clone());
                        },
                        svg {
                            class: "w-5 h-5",
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            path { d: "M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" }
                        }
                    }
                }
            }
        }
    }
}
