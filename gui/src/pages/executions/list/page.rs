use crate::components::{
    Alert, AlertType, BadgeButton, BadgeColor, EmptyState, List, PageHeader, SectionCard,
};
use dioxus::prelude::*;
use s_e_e_core::WorkflowExecutionStatus;

use super::components::{ExecutionItem, RunningWorkflowItem};
use super::hooks::use_execution_list;

#[component]
pub fn ExecutionListPage() -> Element {
    tracing::trace!("rendering execution list page");

    let (executions_result, running_result) = use_execution_list();

    // Check for errors
    if executions_result.is_err() || running_result.is_err() {
        let error_msg = executions_result.err().unwrap_or_else(|| {
            running_result
                .err()
                .unwrap_or_else(|| "Unknown error".to_string())
        });

        return rsx! {
            div { class: "space-y-8",
                PageHeader {
                    title: "Executions".to_string(),
                    description: "View and manage your workflow executions".to_string(),
                    actions: None,
                }
                Alert {
                    alert_type: AlertType::Error,
                    title: Some("Failed to load executions".to_string()),
                    message: error_msg,
                    dismissible: None,
                    on_dismiss: None,
                    actions: None,
                }
            }
        };
    }

    // Get the data
    let workflow_executions = executions_result.unwrap();
    let running_workflows = running_result.unwrap();

    // Determine active filter based on available data; default to All
    let mut active_filter = use_signal(|| None::<WorkflowExecutionStatus>);

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Executions".to_string(),
                description: "View and manage your workflow executions".to_string(),
                actions: None,
            }

            SectionCard {
                title: Some("Executions".to_string()),
                children: rsx! {
                    div { class: "space-y-4",
                        // Filter Badges
                        div { class: "flex items-center gap-2 flex-wrap",
                            // Match Home page filters: All, Waiting for Input, Complete, Running, Failed, Pending
                            BadgeButton {
                                color: BadgeColor::Zinc,
                                active: active_filter().is_none(),
                                onclick: move |_| active_filter.set(None),
                                "All"
                            }
                            BadgeButton {
                                color: BadgeColor::Amber,
                                active: active_filter() == Some(WorkflowExecutionStatus::WaitingForInput),
                                onclick: move |_| active_filter.set(Some(WorkflowExecutionStatus::WaitingForInput)),
                                "Waiting for Input"
                            }
                            BadgeButton {
                                color: BadgeColor::Emerald,
                                active: active_filter() == Some(WorkflowExecutionStatus::Complete),
                                onclick: move |_| active_filter.set(Some(WorkflowExecutionStatus::Complete)),
                                "Complete"
                            }
                            BadgeButton {
                                color: BadgeColor::Blue,
                                active: active_filter() == Some(WorkflowExecutionStatus::Running),
                                onclick: move |_| active_filter.set(Some(WorkflowExecutionStatus::Running)),
                                "Running"
                            }
                            BadgeButton {
                                color: BadgeColor::Red,
                                active: active_filter() == Some(WorkflowExecutionStatus::Failed),
                                onclick: move |_| active_filter.set(Some(WorkflowExecutionStatus::Failed)),
                                "Failed"
                            }
                            BadgeButton {
                                color: BadgeColor::Zinc,
                                active: active_filter() == Some(WorkflowExecutionStatus::Pending),
                                onclick: move |_| active_filter.set(Some(WorkflowExecutionStatus::Pending)),
                                "Pending"
                            }
                        }

                        // Display based on active filter
                        {{
                            // Build filtered list based on active_filter
                            if active_filter() == Some(WorkflowExecutionStatus::Running) {
                                rsx! {
                                    if running_workflows.is_empty() {
                                        EmptyState { message: "No running workflows.".to_string() }
                                    } else {
                                        List {
                                            for workflow in running_workflows.iter() {
                                                RunningWorkflowItem { workflow: workflow.clone() }
                                            }
                                        }
                                    }
                                }
                            } else {
                                let mut items: Vec<_> = match active_filter() {
                                    Some(status) => workflow_executions
                                        .iter()
                                        .filter(|exec| exec.status == status)
                                        .cloned()
                                        .collect(),
                                    None => workflow_executions.iter().cloned().collect(),
                                };
                                items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                                rsx! {
                                    if items.is_empty() {
                                        EmptyState { message: "No executions with this status.".to_string() }
                                    } else {
                                        List {
                                            for execution in items.iter() {
                                                ExecutionItem { execution: execution.clone() }
                                            }
                                        }
                                    }
                                }
                            }
                        }}
                    }
                },
                padding: None,
            }
        }
    }
}
