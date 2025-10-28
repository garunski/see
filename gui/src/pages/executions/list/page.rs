use crate::components::{
    Alert, AlertType, Badge, BadgeButton, BadgeColor, EmptyState, List, PageHeader, SectionCard,
};
use dioxus::prelude::*;
use s_e_e_core::{WorkflowExecutionStatus, WorkflowExecutionSummary, WorkflowMetadata};

use super::components::{HistoryItem, LoadingSkeleton, RunningWorkflowItem};
use super::hooks::use_execution_list;

#[derive(PartialEq, Clone, Copy)]
pub enum ExecutionFilter {
    Running,
    WaitingForInput,
    Completed,
}

#[component]
pub fn ExecutionListPage() -> Element {
    tracing::debug!("rendering execution list page");

    let (history_result, running_result) = use_execution_list();

    // Check for errors
    if history_result.is_err() || running_result.is_err() {
        let error_msg = history_result.err().unwrap_or_else(|| {
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
    let workflow_history = history_result.unwrap();
    let running_workflows = running_result.unwrap();

    // Separate workflows into categories
    let waiting_list: Vec<_> = workflow_history
        .iter()
        .filter(|exec| exec.status == WorkflowExecutionStatus::WaitingForInput)
        .cloned()
        .collect();

    let completed_list: Vec<_> = workflow_history
        .iter()
        .filter(|exec| exec.status == WorkflowExecutionStatus::Complete)
        .cloned()
        .collect();

    // Determine active filter based on available data
    let mut active_filter = use_signal(|| {
        if !running_workflows.is_empty() {
            ExecutionFilter::Running
        } else if !waiting_list.is_empty() {
            ExecutionFilter::WaitingForInput
        } else {
            ExecutionFilter::Completed
        }
    });

    // Get counts for badges
    let running_count = running_workflows.len();
    let waiting_count = waiting_list.len();
    let completed_count = completed_list.len();

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
                            BadgeButton {
                                color: BadgeColor::Blue,
                                active: active_filter() == ExecutionFilter::Running,
                                onclick: move |_| active_filter.set(ExecutionFilter::Running),
                                "Running"
                            }
                            BadgeButton {
                                color: BadgeColor::Amber,
                                active: active_filter() == ExecutionFilter::WaitingForInput,
                                onclick: move |_| active_filter.set(ExecutionFilter::WaitingForInput),
                                "Waiting for Input"
                            }
                            BadgeButton {
                                color: BadgeColor::Emerald,
                                active: active_filter() == ExecutionFilter::Completed,
                                onclick: move |_| active_filter.set(ExecutionFilter::Completed),
                                "Completed"
                            }
                        }

                        // Display based on active filter
                        {match active_filter() {
                            ExecutionFilter::Running => rsx! {
                                if running_workflows.is_empty() {
                                    EmptyState {
                                        message: "No running workflows.".to_string(),
                                    }
                                } else {
                                    List {
                                        for workflow in running_workflows.iter() {
                                            RunningWorkflowItem {
                                                workflow: workflow.clone(),
                                            }
                                        }
                                    }
                                }
                            },
                            ExecutionFilter::WaitingForInput => rsx! {
                                if waiting_list.is_empty() {
                                    EmptyState {
                                        message: "No workflows waiting for input.".to_string(),
                                    }
                                } else {
                                    List {
                                        for execution in waiting_list.iter() {
                                            HistoryItem {
                                                execution: execution.clone(),
                                            }
                                        }
                                    }
                                }
                            },
                            ExecutionFilter::Completed => rsx! {
                                if completed_list.is_empty() {
                                    EmptyState {
                                        message: "No completed workflows yet.".to_string(),
                                    }
                                } else {
                                    List {
                                        for execution in completed_list.iter() {
                                            HistoryItem {
                                                execution: execution.clone(),
                                            }
                                        }
                                    }
                                }
                            },
                        }}
                    }
                },
                padding: None,
            }
        }
    }
}
