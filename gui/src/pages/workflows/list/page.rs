use crate::components::{EmptyState, List, ListItemWithLink, PageHeader, SectionCard};
use crate::hooks::{use_system_workflows, use_workflows};
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use s_e_e_core::SystemWorkflow;

#[component]
pub fn WorkflowsListPage() -> Element {
    let workflows = use_workflows();
    let system_workflows = use_system_workflows();

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

            // System Workflows Section
            if !system_workflows().is_empty() {
                SectionCard {
                    title: Some("System Templates".to_string()),
                    children: rsx! {
                        List {
                            for idx in 0..system_workflows().len() {
                                SystemWorkflowItem {
                                    workflow: system_workflows()[idx].clone()
                                }
                            }
                        }
                    },
                    padding: None,
                }
            }

            // User Workflows Section
            if workflows().is_empty() {
                SectionCard {
                    title: Some("My Workflows".to_string()),
                    children: rsx! {
                        EmptyState {
                            message: "No workflows yet. Clone a system template or create your first workflow to get started.".to_string(),
                        }
                    },
                    padding: None,
                }
            } else {
                SectionCard {
                    title: Some("My Workflows".to_string()),
                    children: rsx! {
                        List {
                            for workflow in workflows().iter() {
                                ListItemWithLink {
                                    icon_name: "workflows".to_string(),
                                    icon_variant: Some("outline".to_string()),
                                    title: workflow.get_name().to_string(),
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
                                    link_to: rsx! {
                                        Link {
                                            to: Route::WorkflowEditPage { id: workflow.id.clone() },
                                            span { class: "absolute inset-x-0 -top-px bottom-0" }
                                            {workflow.get_name().to_string()}
                                        }
                                    },
                                }
                            }
                        }
                    },
                    padding: None,
                }
            }
        }
    }
}

#[component]
fn SystemWorkflowItem(workflow: SystemWorkflow) -> Element {
    let workflow_id = workflow.id.clone();
    let workflow_name_display = workflow.name.clone();
    let workflow_name_link = workflow.name.clone();

    rsx! {
        ListItemWithLink {
            icon_name: "workflows".to_string(),
            icon_variant: Some("outline".to_string()),
            title: workflow_name_display,
            subtitle: Some(rsx! {
                span { class: "inline-flex items-center rounded-md bg-purple-50 dark:bg-purple-900/20 px-2 py-1 text-xs font-medium text-purple-700 dark:text-purple-300 ring-1 ring-inset ring-purple-700/10",
                    "System"
                }
            }),
            right_content: None,
            link_to: rsx! {
                Link {
                    to: Route::WorkflowEditPage { id: workflow_id },
                    span { class: "absolute inset-x-0 -top-px bottom-0" }
                    {workflow_name_link}
                }
            },
        }
    }
}
