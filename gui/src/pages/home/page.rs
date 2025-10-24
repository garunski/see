use crate::components::{List, PageHeader};
use crate::hooks::use_workflows;
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use s_e_e_core::WorkflowDefinition;

use super::components::{ActionCard, ActionIcon};
use super::hooks::use_workflow_execution;

#[component]
pub fn WorkflowExecutionItem(workflow: WorkflowDefinition) -> Element {
    let execute_workflow = use_workflow_execution();
    let navigator = use_navigator();

    rsx! {
        li {
            class: "relative flex justify-between gap-x-6 px-4 py-5 hover:bg-gray-50 sm:px-6 dark:hover:bg-white/[0.025] cursor-pointer",
            onclick: move |_| {
                let workflow_name = workflow.get_name();
                let workflow_id = workflow.id.clone();
                execute_workflow(workflow_name, workflow_id);
                navigator.push(Route::HistoryPage {});
            },
            div { class: "flex min-w-0 gap-x-4",
                div { class: "size-12 flex-none rounded-full bg-gray-50 dark:bg-gray-800 dark:outline dark:outline-1 dark:-outline-offset-1 dark:outline-white/10 flex items-center justify-center",
                    Icon {
                        name: "play".to_string(),
                        class: Some("size-6 text-gray-400 dark:text-gray-500".to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                }
                div { class: "min-w-0 flex-auto",
                    p { class: "text-sm/6 font-semibold text-gray-900 dark:text-white",
                        {workflow.get_name()}
                    }
                    div { class: "mt-1 flex items-center gap-2",
                        if workflow.is_default {
                            span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                                "Default"
                            }
                        } else {
                            span { class: "inline-flex items-center rounded-md bg-gray-50 dark:bg-gray-800 px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-300 ring-1 ring-inset ring-gray-500/10",
                                "Custom"
                            }
                        }
                        if workflow.is_default && workflow.is_edited {
                            span { class: "inline-flex items-center rounded-md bg-yellow-50 dark:bg-yellow-900/20 px-2 py-1 text-xs font-medium text-yellow-700 dark:text-yellow-300 ring-1 ring-inset ring-yellow-600/10",
                                "Modified"
                            }
                        }
                    }
                }
            }
            div { class: "flex shrink-0 items-center gap-x-4",
                div { class: "hidden sm:flex sm:flex-col sm:items-end",
                    span { class: "inline-flex items-center rounded-md bg-green-50 dark:bg-green-900/20 px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-600/10",
                        "Ready to Run"
                    }
                }
                Icon {
                    name: "chevron_right".to_string(),
                    class: Some("size-5 flex-none text-gray-400 dark:text-gray-500".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
            }
        }
    }
}

#[component]
pub fn HomePage() -> Element {
    let workflows = use_workflows();

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Welcome to Speculative Execution Engine".to_string(),
                description: "Your workflow automation platform".to_string(),
                actions: None,
            }

            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                ActionCard {
                    title: "Upload".to_string(),
                    description: "Upload workflow files".to_string(),
                    icon: ActionIcon::Upload,
                    route: Route::UploadPage {},
                }

                ActionCard {
                    title: "Workflows".to_string(),
                    description: "Edit and organize".to_string(),
                    icon: ActionIcon::Workflows,
                    route: Route::WorkflowsListPage {},
                }

                ActionCard {
                    title: "History".to_string(),
                    description: "View execution logs".to_string(),
                    icon: ActionIcon::History,
                    route: Route::HistoryPage {},
                }
            }

            div { class: "space-y-4",
                h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Execute Workflows" }

                if workflows().is_empty() {
                    div { class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-8 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No workflows yet. Create your first workflow to get started."
                        }
                    }
                } else {
                    List {
                        for workflow in workflows().iter().take(6) {
                            WorkflowExecutionItem {
                                workflow: workflow.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}
