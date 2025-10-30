use crate::components::layout::ListItem as LayoutListItem;
use crate::components::{BadgeButton, BadgeColor, EmptyState, List, PageHeader, SectionCard};
use crate::queries::{
    use_execute_workflow_mutation, use_workflow_executions_query, use_workflows_query,
};
use dioxus::prelude::*;
use s_e_e_core::WorkflowExecutionStatus;

use super::components::ExecutionListItem;

#[component]
pub fn HomePage() -> Element {
    let (workflows_state, _refetch_workflows) = use_workflows_query();
    let (executions_state, _refetch_executions) = use_workflow_executions_query();
    let (_exec_mutation_state, execute_fn) = use_execute_workflow_mutation();

    let workflows = if workflows_state.is_loading {
        return rsx! {
            div { class: "flex items-center justify-center h-64",
                "Loading workflows..."
            }
        };
    } else if workflows_state.is_error {
        let e = workflows_state.error.as_deref().unwrap_or("Unknown error");
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
    } else {
        workflows_state.data.clone().unwrap_or_default()
    };

    let workflow_executions = if executions_state.is_loading {
        vec![]
    } else if executions_state.is_error {
        let e = executions_state.error.as_deref().unwrap_or("Unknown error");
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
    } else {
        executions_state.data.clone().unwrap_or_default()
    };

    let mut active_filter = use_signal(|| None::<WorkflowExecutionStatus>);

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Welcome to Speculative Execution Engine".to_string(),
                description: "Your workflow automation platform".to_string(),
                actions: None,
            }


            SectionCard {
                title: Some("Recent Executions".to_string()),
                children: rsx! {
                    {{

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

                                div { class: "flex items-center gap-2 flex-wrap",

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
                              {
                                  let workflow_id = workflow.id.clone();
                                  let execute_fn = execute_fn.clone();
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
                                              execute_fn(workflow_id.clone());
                                              tracing::debug!("[HomePage] Execution started");
                                          },
                                      }
                                  }
                              }
                          }
                    }
                }
            }
        }
    }
}
