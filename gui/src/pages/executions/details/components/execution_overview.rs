use crate::icons::Icon;
use dioxus::prelude::*;
use s_e_e_core::{WorkflowExecution, WorkflowExecutionStatus};

#[component]
pub fn ExecutionOverview(execution: WorkflowExecution) -> Element {
    // Count pending inputs
    let pending_input_count = execution
        .tasks
        .iter()
        .filter(|t| t.status.as_str() == "waiting_for_input")
        .count();

    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{execution.workflow_name}" }
                div { class: "flex items-center gap-3",
                    if pending_input_count > 0 {
                        div {
                            class: "flex items-center gap-2 px-3 py-1 bg-amber-100 dark:bg-amber-900/30 rounded-full",
                            Icon {
                                name: "pause".to_string(),
                                class: Some("text-amber-600 dark:text-amber-400".to_string()),
                                size: Some("w-4 h-4".to_string()),
                                variant: Some("outline".to_string()),
                            }
                            span {
                                class: "text-sm font-medium text-amber-800 dark:text-amber-200",
                                if pending_input_count == 1 {
                                    "1 input required"
                                } else {
                                    "{pending_input_count} inputs required"
                                }
                            }
                        }
                    }
                    div {
                        class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                            match execution.status {
                                WorkflowExecutionStatus::WaitingForInput => "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
                                WorkflowExecutionStatus::Complete => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                WorkflowExecutionStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                WorkflowExecutionStatus::Running => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                WorkflowExecutionStatus::Pending => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                            }
                        ),
                        match execution.status {
                            WorkflowExecutionStatus::WaitingForInput => "Waiting for Input",
                            WorkflowExecutionStatus::Complete => "Success",
                            WorkflowExecutionStatus::Failed => "Failed",
                            WorkflowExecutionStatus::Running => "Running",
                            WorkflowExecutionStatus::Pending => "Pending",
                        }
                    }
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div {
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Execution ID" }
                    div { class: "text-zinc-950 dark:text-white font-mono text-sm", "{execution.id}" }
                }
                div {
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Executed At" }
                    div { class: "text-zinc-950 dark:text-white", "{execution.timestamp}" }
                }
                div {
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Tasks" }
                    div { class: "text-zinc-950 dark:text-white", "{execution.tasks.len()} total" }
                }
            }
        }
    }
}
