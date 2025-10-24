use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::WorkflowDefinition;

use super::hooks::use_workflow_execution;

#[derive(Props, PartialEq, Clone)]
pub struct WorkflowCardProps {
    pub workflow: WorkflowDefinition,
}

#[component]
pub fn WorkflowCard(props: WorkflowCardProps) -> Element {
    let WorkflowCardProps { workflow } = props;
    let execute_workflow = use_workflow_execution();

    rsx! {
        div {
            class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-4 hover:bg-zinc-50 dark:hover:bg-zinc-700 hover:shadow-md transition-colors cursor-pointer",
            onclick: move |_| {
                let workflow_id = workflow.id.clone();
                let workflow_name = workflow.get_name();
                execute_workflow(workflow_name, workflow_id);
            },
            div { class: "flex items-start justify-between",
                div { class: "flex-1 min-w-0",
                    h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white truncate",
                        {workflow.get_name()}
                    }
                    div { class: "mt-1 flex items-center gap-2",
                        if workflow.is_default {
                            span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                                "Default"
                            }
                        } else {
                            span { class: "inline-flex items-center rounded-md bg-zinc-50 dark:bg-zinc-800 px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 ring-1 ring-inset ring-zinc-500/10",
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
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ActionCardProps {
    pub title: String,
    pub description: String,
    pub icon: ActionIcon,
    pub route: Route,
}

#[derive(PartialEq, Clone)]
pub enum ActionIcon {
    Plus,
    Upload,
    History,
}

#[component]
pub fn ActionCard(props: ActionCardProps) -> Element {
    let ActionCardProps {
        title,
        description,
        icon,
        route,
    } = props;

    let (icon_name, icon_class) = match icon {
        ActionIcon::Plus => ("plus", "h-5 w-5 text-blue-600 dark:text-blue-400"),
        ActionIcon::Upload => ("upload", "h-5 w-5 text-green-600 dark:text-green-400"),
        ActionIcon::History => ("history", "h-5 w-5 text-purple-600 dark:text-purple-400"),
    };

    let bg_class = match icon {
        ActionIcon::Plus => "bg-blue-100 dark:bg-blue-900/20",
        ActionIcon::Upload => "bg-green-100 dark:bg-green-900/20",
        ActionIcon::History => "bg-purple-100 dark:bg-purple-900/20",
    };

    rsx! {
        Link {
            to: route,
            class: "group relative rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-6 hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            div { class: "flex items-center gap-3",
                div { class: "flex h-10 w-10 items-center justify-center rounded-lg {bg_class}",
                    Icon {
                        name: icon_name.to_string(),
                        class: Some(icon_class.to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                }
                div {
                    h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white", "{title}" }
                    p { class: "text-xs text-zinc-500 dark:text-zinc-400", "{description}" }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct WorkflowListProps {
    pub workflows: Vec<WorkflowDefinition>,
}

#[component]
pub fn WorkflowList(props: WorkflowListProps) -> Element {
    let WorkflowListProps { workflows } = props;

    rsx! {
        if workflows.is_empty() {
            div { class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-8 text-center",
                div { class: "text-zinc-500 dark:text-zinc-400",
                    "No workflows yet. Create your first workflow to get started."
                }
            }
        } else {
            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                for workflow in workflows.iter().take(6) {
                    WorkflowCard {
                        workflow: workflow.clone(),
                    }
                }
            }
        }
    }
}
