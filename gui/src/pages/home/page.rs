use crate::components::layout::ListItem as LayoutListItem;
use crate::components::{BadgeButton, BadgeColor, EmptyState, List, PageHeader, SectionCard};
use crate::queries::{ExecuteWorkflowMutation, GetWorkflowExecutions, GetWorkflows};
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use s_e_e_core::WorkflowExecutionStatus;

use super::components::ExecutionListItem;

#[component]
pub fn HomePage() -> Element {
    // Direct query calls following best practices
    let workflows_query = use_query(Query::new((), GetWorkflows));
    let workflow_executions_query = use_query(
        Query::new((), GetWorkflowExecutions).interval_time(std::time::Duration::from_secs(1)),
    );

    // Automatic polling is handled by dioxus-query's interval_time above

    // Mutation for executing workflows
    let execute_mutation = use_mutation(Mutation::new(ExecuteWorkflowMutation));

    // Handle workflows query result
    let workflows = match workflows_query.suspend() {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Welcome to Speculative Execution Engine".to_string(),
                        description: "Your workflow automation platform".to_string(),
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
        Err(e) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Welcome to Speculative Execution Engine".to_string(),
                        description: "Your workflow automation platform".to_string(),
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

    // Handle workflow executions query result
    let workflow_executions = match workflow_executions_query.suspend() {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Welcome to Speculative Execution Engine".to_string(),
                        description: "Your workflow automation platform".to_string(),
                        actions: None,
                    }
                    SectionCard {
                        title: Some("Error".to_string()),
                        children: rsx! {
                            div { class: "text-red-600 dark:text-red-400",
                                "Failed to load workflow executions: {e}"
                            }
                        },
                        padding: None,
                    }
                }
            };
        }
        Err(e) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Welcome to Speculative Execution Engine".to_string(),
                        description: "Your workflow automation platform".to_string(),
                        actions: None,
                    }
                    SectionCard {
                        title: Some("Error".to_string()),
                        children: rsx! {
                            div { class: "text-red-600 dark:text-red-400",
                                "Failed to load workflow executions: {e}"
                            }
                        },
                        padding: None,
                    }
                }
            };
        }
    };

    // Filter state (None = All). Default to All
    let mut active_filter = use_signal(|| None::<WorkflowExecutionStatus>);

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Welcome to Speculative Execution Engine".to_string(),
                description: "Your workflow automation platform".to_string(),
                actions: None,
            }

            // Executions Section
            SectionCard {
                title: Some("Recent Executions".to_string()),
                children: rsx! {
                    {{
                        // Filter and sort executions inline so it reacts to workflow_executions changes
                        let mut items: Vec<_> = match active_filter() {
                            Some(status) => workflow_executions
                                .iter()
                                .filter(|exec| exec.status == status)
                                .cloned()
                                .collect(),
                            None => workflow_executions.to_vec(),
                        };

                        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                        let sorted: Vec<_> = items.into_iter().take(5).collect();

                        rsx! {
                            div { class: "space-y-4",
                                // Filter Badges
                                div { class: "flex items-center gap-2 flex-wrap",
                                    // All badge (clears filter)
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

                                if sorted.is_empty() {
                                    EmptyState { message: "No executions with this status.".to_string() }
                                } else {
                                    List {
                                        for execution in sorted.iter() {
                                            ExecutionListItem { execution: execution.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }}
                },
                padding: None,
            }

            // Execute Workflows Section
            div { class: "space-y-4",
                h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Execute Workflows" }

                if workflows.is_empty() {
                    div { class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-8 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No workflows yet. Create your first workflow to get started."
                        }
                    }
                } else {
                    List {
                        for workflow in workflows.iter().take(6) {
                            {let workflow_id = workflow.id.clone();
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
                                        tracing::debug!("[HomePage] Clicked workflow: {}", workflow_id);
                                        let workflow_id_clone = workflow_id.clone();
                                        execute_mutation.mutate(workflow_id_clone);
                                        tracing::debug!("[HomePage] Execution started");
                                    },
                                }
                            }}
                        }
                    }
                }
            }
        }
    }
}
