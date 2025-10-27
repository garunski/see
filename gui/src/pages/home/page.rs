use crate::components::layout::ListItem as LayoutListItem;
use crate::components::{BadgeButton, BadgeColor, List, PageHeader};
use crate::hooks::{use_workflow_history, use_workflows};
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};
use s_e_e_core::{WorkflowDefinition, WorkflowExecutionStatus};

use super::components::ExecutionListItem;
use super::hooks::use_workflow_execution;

#[component]
pub fn WorkflowExecutionItem(workflow: WorkflowDefinition) -> Element {
    let execute_workflow = use_workflow_execution();
    let navigator = use_navigator();

    rsx! {
        LayoutListItem {
            icon_name: "play".to_string(),
            icon_variant: Some("outline".to_string()),
            title: rsx! {
                {workflow.get_name().to_string()}
            },
            subtitle: Some(rsx! {
                div { class: "flex items-center gap-2",
                    if workflow.is_default {
                        span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                            "Default"
                        }
                    } else {
                        span { class: "inline-flex items-center rounded-md bg-gray-50 dark:bg-gray-800 px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-300 ring-1 ring-inset ring-gray-500/10",
                            "Custom"
                        }
                    }
                    if workflow.is_default && workflow.is_edited {
                        span { class: "inline-flex items-center rounded-md bg-yellow-50 dark:bg-yellow-900/20 px-2 py-1 text-xs font-medium text-yellow-700 dark:text-yellow-300 ring-1 ring-inset ring-yellow-600/10",
                            "Modified"
                        }
                    }
                }
            }),
            right_content: Some(rsx! {
                span { class: "inline-flex items-center rounded-md bg-green-50 dark:bg-green-900/20 px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-600/10",
                    "Ready to Run"
                }
            }),
            onclick: move |_| {
                let workflow_name = workflow.get_name().to_string();
                let workflow_id = workflow.id.clone();
                execute_workflow(workflow_name.to_string(), workflow_id);
                navigator.push(Route::HistoryPage {});
            },
        }
    }
}

#[component]
pub fn HomePage() -> Element {
    let workflows = use_workflows();
    let workflow_history = use_workflow_history();

    // Filter state
    let mut active_filter = use_signal(|| {
        let history = workflow_history();
        let has_waiting = history
            .iter()
            .any(|exec| matches!(exec.status, WorkflowExecutionStatus::WaitingForInput));

        if has_waiting {
            WorkflowExecutionStatus::WaitingForInput
        } else {
            WorkflowExecutionStatus::Complete
        }
    });

    // Filter executions based on active filter
    let filtered_executions = use_memo(move || {
        let mut filtered: Vec<_> = workflow_history()
            .iter()
            .filter(|exec| exec.status == active_filter())
            .cloned()
            .collect();

        // Sort by created_at descending
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        filtered.into_iter().take(5).collect::<Vec<_>>()
    });

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Welcome to Speculative Execution Engine".to_string(),
                description: "Your workflow automation platform".to_string(),
                actions: None,
            }

            // Executions Section
            div { class: "space-y-4",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Recent Executions" }
                    Link {
                        to: Route::HistoryPage {},
                        class: "text-sm text-blue-600 hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300",
                        "View All"
                    }
                }

                // Filter Badges
                div { class: "flex items-center gap-2 flex-wrap",
                    BadgeButton {
                        color: BadgeColor::Amber,
                        active: active_filter() == WorkflowExecutionStatus::WaitingForInput,
                        onclick: move |_| active_filter.set(WorkflowExecutionStatus::WaitingForInput),
                        "Waiting for Input"
                    }
                    BadgeButton {
                        color: BadgeColor::Emerald,
                        active: active_filter() == WorkflowExecutionStatus::Complete,
                        onclick: move |_| active_filter.set(WorkflowExecutionStatus::Complete),
                        "Complete"
                    }
                    BadgeButton {
                        color: BadgeColor::Blue,
                        active: active_filter() == WorkflowExecutionStatus::Running,
                        onclick: move |_| active_filter.set(WorkflowExecutionStatus::Running),
                        "Running"
                    }
                    BadgeButton {
                        color: BadgeColor::Red,
                        active: active_filter() == WorkflowExecutionStatus::Failed,
                        onclick: move |_| active_filter.set(WorkflowExecutionStatus::Failed),
                        "Failed"
                    }
                    BadgeButton {
                        color: BadgeColor::Zinc,
                        active: active_filter() == WorkflowExecutionStatus::Pending,
                        onclick: move |_| active_filter.set(WorkflowExecutionStatus::Pending),
                        "Pending"
                    }
                }

                if filtered_executions().is_empty() {
                    div { class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-8 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No executions with this status."
                        }
                    }
                } else {
                    List {
                        for execution in filtered_executions().iter() {
                            ExecutionListItem {
                                execution: execution.clone(),
                            }
                        }
                    }
                }
            }

            // Execute Workflows Section
            div { class: "space-y-4",
                h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Execute Workflows" }

                if workflows().is_empty() {
                    div { class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-8 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No workflows yet. Create your first workflow to get started."
                        }
                    }
                } else {
                    List {
                        for workflow in workflows().iter().take(6) {
                            WorkflowExecutionItem {
                                workflow: workflow.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}
