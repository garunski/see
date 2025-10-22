use super::super::utils::get_status_badge_variant;
use crate::components::ui::{Badge, Card};
use dioxus::prelude::*;
use see_core::WorkflowExecution;

#[derive(Props, PartialEq, Clone)]
pub struct ExecutionSummaryProps {
    pub execution: WorkflowExecution,
}

#[component]
pub fn ExecutionSummary(props: ExecutionSummaryProps) -> Element {
    let exec = &props.execution;
    let status_variant = get_status_badge_variant(exec.success);

    rsx! {
        Card {
            div {
                class: "flex items-center justify-between mb-4",
                h2 {
                    class: "text-base font-semibold text-zinc-950 dark:text-white",
                    {exec.workflow_name.clone()}
                }
                Badge {
                    variant: status_variant,
                    if exec.success { "Success" } else { "Failed" }
                }
            }
            div {
                class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div {
                    div {
                        class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Execution ID"
                    }
                    div {
                        class: "text-zinc-950 dark:text-white font-mono text-sm",
                        {exec.id.clone()}
                    }
                }
                div {
                    div {
                        class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Executed At"
                    }
                    div {
                        class: "text-zinc-950 dark:text-white",
                        {exec.timestamp.clone()}
                    }
                }
                div {
                    div {
                        class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Tasks"
                    }
                    div {
                        class: "text-zinc-950 dark:text-white",
                        "{exec.tasks.len()} total"
                    }
                }
            }
        }
    }
}
