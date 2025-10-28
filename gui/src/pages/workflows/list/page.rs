use crate::components::layout::{List, ListItem};
use crate::components::{EmptyState, PageHeader, SectionCard};
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};

use super::hooks::use_workflows_list;

#[component]
pub fn WorkflowsListPage() -> Element {
    let workflows = match use_workflows_list() {
        Ok(w) => w,
        Err(e) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Workflows".to_string(),
                        description: "Manage your workflow definitions".to_string(),
                        actions: None,
                    }
                    SectionCard {
                        title: Some("Error".to_string()),
                        children: rsx! {
                            div { class: "text-red-600 dark:text-red-400",
                                "Failed to load workflows: {e}"
                            }
                        },
                        padding: None,
                    }
                }
            };
        }
    };

    let navigator = use_navigator();

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

            // Workflows Section
            if workflows.is_empty() {
                SectionCard {
                    title: Some("Workflows".to_string()),
                    children: rsx! {
                        EmptyState {
                            message: "No workflows yet. Create your first workflow to get started.".to_string(),
                        }
                    },
                    padding: None,
                }
            } else {
                SectionCard {
                    title: Some("Workflows".to_string()),
                    children: rsx! {
                        List {
                            for workflow in workflows.iter() {
                                {let workflow_id = workflow.id.clone();
                                rsx! {
                                    ListItem {
                                        icon_name: "workflows".to_string(),
                                        icon_variant: Some("outline".to_string()),
                                        title: rsx! {
                                            {workflow.get_name().to_string()}
                                        },
                                        subtitle: Some(rsx! {
                                            if workflow.is_default {
                                                span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                                                    "Default"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center rounded-md bg-gray-50 dark:bg-gray-800 px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-300 ring-1 ring-inset ring-gray-500/10",
                                                    "Custom"
                                                }
                                            }
                                        }),
                                        right_content: Some(rsx! {
                                            if workflow.is_default && workflow.is_edited {
                                                span { class: "inline-flex items-center rounded-md bg-yellow-50 dark:bg-yellow-900/20 px-2 py-1 text-xs font-medium text-yellow-700 dark:text-yellow-300 ring-1 ring-inset ring-yellow-600/10",
                                                    "Modified"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center rounded-md bg-green-50 dark:bg-green-900/20 px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-600/10",
                                                    "Active"
                                                }
                                            }
                                        }),
                                        onclick: move |_| {
                                            navigator.push(Route::WorkflowEditPage { id: workflow_id.clone() });
                                        },
                                    }
                                }}
                            }
                        }
                    },
                    padding: None,
                }
            }
        }
    }
}
