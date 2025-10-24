use dioxus::prelude::*;
use see_core::WorkflowDefinition;

use super::WorkflowCard;

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
