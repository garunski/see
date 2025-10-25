use dioxus::prelude::*;
use s_e_e_core::WorkflowDefinition;

use crate::pages::home::hooks::use_workflow_execution;

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
                let workflow_name = workflow.get_name().to_string();
                execute_workflow(workflow_name, workflow_id);
            },
            div { class: "flex items-start justify-between",
                div { class: "flex-1 min-w-0",
                    h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white truncate",
                        {workflow.get_name().to_string()}
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
