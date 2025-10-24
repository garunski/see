use crate::components::{EmptyState, PageHeader, SectionCard};
use crate::hooks::use_workflows;
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn WorkflowsListPage() -> Element {
    let workflows = use_workflows();

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Workflows".to_string(),
                description: "Manage your workflow definitions".to_string(),
                actions: Some(rsx! {
                    Link {
                        to: Route::WorkflowEditPageNew {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                        Icon {
                            name: "plus".to_string(),
                            class: Some("-ml-0.5 h-5 w-5".to_string()),
                            size: None,
                            variant: Some("outline".to_string()),
                        }
                        "Create workflow"
                    }
                }),
            }

            SectionCard {
                title: Some("All Workflows".to_string()),
                children: rsx! {
                    if workflows().is_empty() {
                        EmptyState {
                            message: "No workflows yet. Create your first workflow to get started.".to_string(),
                        }
                    } else {
                        div { class: "overflow-hidden",
                            table { class: "min-w-full divide-y divide-zinc-200 dark:divide-zinc-700",
                                thead { class: "bg-zinc-50 dark:bg-zinc-700",
                                    tr {
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Name" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Type" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Status" }
                                        th { class: "px-6 py-3 text-right text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Actions" }
                                    }
                                }
                                tbody { class: "bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700",
                                    for workflow in workflows().iter() {
                                        tr { class: "hover:bg-zinc-50 dark:hover:bg-zinc-700",
                                            td { class: "px-6 py-4 whitespace-nowrap",
                                                div { class: "text-sm font-medium text-zinc-900 dark:text-white",
                                                    {workflow.get_name()}
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap",
                                                if workflow.is_default {
                                                    span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                                                        "Default"
                                                    }
                                                } else {
                                                    span { class: "inline-flex items-center rounded-md bg-zinc-50 dark:bg-zinc-800 px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 ring-1 ring-inset ring-zinc-500/10",
                                                        "Custom"
                                                    }
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap",
                                                if workflow.is_default && workflow.is_edited {
                                                    span { class: "inline-flex items-center rounded-md bg-yellow-50 dark:bg-yellow-900/20 px-2 py-1 text-xs font-medium text-yellow-700 dark:text-yellow-300 ring-1 ring-inset ring-yellow-600/10",
                                                        "Modified"
                                                    }
                                                } else {
                                                    span { class: "inline-flex items-center rounded-md bg-green-50 dark:bg-green-900/20 px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-600/10",
                                                        "Active"
                                                    }
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                                                div { class: "flex items-center justify-end gap-3",
                                                    Link {
                                                        to: Route::WorkflowEditPage { id: workflow.id.clone() },
                                                        class: "text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300",
                                                        "Edit"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                padding: None,
            }
        }
    }
}
