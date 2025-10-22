use crate::router::Route;
use crate::services::workflow::{create_output_channel, run_workflow_from_content};
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use see_core::WorkflowDefinition;
use std::sync::Arc;

#[component]
fn WorkflowCard(workflow: WorkflowDefinition) -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let store = use_context::<Option<Arc<see_core::RedbStore>>>();
    let navigator = use_navigator();

    rsx! {
        div {
            class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-4 hover:bg-zinc-50 dark:hover:bg-zinc-700 hover:shadow-md transition-colors cursor-pointer",
            onclick: move |_| {
                let workflow_content = workflow.content.clone();
                let workflow_name = workflow.name.clone();
                let store_clone = store.clone();
                let mut workflow_state = state_provider.workflow;
                let mut ui_state = state_provider.ui;
                let mut history_state = state_provider.history;
                let navigator_clone = navigator;

                // Show starting status
                ui_state.write().show_status(
                    format!("Starting workflow: {}", workflow_name),
                    crate::components::ExecutionStatus::Running
                );

                spawn(async move {
                    workflow_state.write().reset_before_run();

                    // Polling will start once we receive EXECUTION_ID from engine output

                    let (output_callback, mut handles) = create_output_channel();

                    let mut workflow_state_clone = workflow_state;
                    spawn(async move {
                        while let Some(msg) = handles.receiver.recv().await {
                            if msg.starts_with("EXECUTION_ID:") {
                                let execution_id = msg
                                    .strip_prefix("EXECUTION_ID:")
                                    .unwrap_or("")
                                    .trim()
                                    .to_string();
                                if !execution_id.is_empty() {
                                    workflow_state_clone.write().execution_id = Some(execution_id.clone());
                                    workflow_state_clone.write().start_polling(execution_id);
                                }
                            } else {
                                workflow_state_clone.write().output_logs.push(msg);
                            }
                        }
                    });

                    match run_workflow_from_content(
                        workflow_content.clone(),
                        output_callback,
                        store_clone.map(|s| s as Arc<dyn see_core::AuditStore>),
                    )
                    .await
                    {
                        Ok(result) => {
                            // Stop polling
                            workflow_state.write().stop_polling();

                            workflow_state.write().apply_success(&result);
                            ui_state
                                .write()
                                .show_status("Workflow completed successfully!".to_string(), crate::components::ExecutionStatus::Complete);
                            history_state.write().needs_history_reload = true;

                            // Navigate to workflow details page
                            navigator_clone.push(Route::WorkflowDetailsPage {
                                id: result.execution_id.clone()
                            });
                        }
                        Err(e) => {
                            // Stop polling on error
                            workflow_state.write().stop_polling();

                            workflow_state.write().apply_failure(&e.to_string());
                            ui_state
                                .write()
                                .show_status(format!("Workflow failed: {}", e), crate::components::ExecutionStatus::Failed);
                        }
                    }
                });
            },
            div { class: "flex items-start justify-between",
                div { class: "flex-1 min-w-0",
                    h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white truncate",
                        {workflow.name.clone()}
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

#[component]
pub fn HomePage() -> Element {
    let state_provider = use_context::<AppStateProvider>();

    // Use a safer approach - read directly from the signal without use_memo
    let workflows = state_provider.settings.read().get_workflows().clone();

    rsx! {
        div { class: "space-y-8",
            // Header
            div {
                h1 { class: "text-2xl font-bold text-zinc-900 dark:text-white", "Welcome to See" }
                p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "Your workflow automation platform" }
            }

            // Quick Actions
            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                Link {
                    to: Route::WorkflowEditPageNew {},
                    class: "group relative rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-6 hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-10 w-10 items-center justify-center rounded-lg bg-blue-100 dark:bg-blue-900/20",
                            svg { class: "h-5 w-5 text-blue-600 dark:text-blue-400", view_box: "0 0 20 20", fill: "currentColor",
                                path { d: "M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z" }
                            }
                        }
                        div {
                            h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white", "Create Workflow" }
                            p { class: "text-xs text-zinc-500 dark:text-zinc-400", "Design a new workflow" }
                        }
                    }
                }

                Link {
                    to: Route::UploadPage {},
                    class: "group relative rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-6 hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-10 w-10 items-center justify-center rounded-lg bg-green-100 dark:bg-green-900/20",
                            svg { class: "h-5 w-5 text-green-600 dark:text-green-400", view_box: "0 0 20 20", fill: "currentColor",
                                path { d: "M10.75 2.75a.75.75 0 00-1.5 0v8.614L6.295 8.235a.75.75 0 10-1.09 1.03l4.25 4.5a.75.75 0 001.09 0l4.25-4.5a.75.75 0 00-1.09-1.03L10.75 11.364V2.75z" }
                            }
                        }
                        div {
                            h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white", "Upload & Execute" }
                            p { class: "text-xs text-zinc-500 dark:text-zinc-400", "Run a workflow file" }
                        }
                    }
                }

                Link {
                    to: Route::HistoryPage {},
                    class: "group relative rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-6 hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-10 w-10 items-center justify-center rounded-lg bg-purple-100 dark:bg-purple-900/20",
                            svg { class: "h-5 w-5 text-purple-600 dark:text-purple-400", view_box: "0 0 20 20", fill: "currentColor",
                                path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                            }
                        }
                        div {
                            h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white", "View History" }
                            p { class: "text-xs text-zinc-500 dark:text-zinc-400", "Check execution logs" }
                        }
                    }
                }
            }

            // Workflows List
            div { class: "space-y-4",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Your Workflows" }
                    Link {
                        to: Route::WorkflowsListPage {},
                        class: "text-sm text-blue-600 dark:text-blue-400 hover:text-blue-900 dark:hover:text-blue-300",
                        "View all"
                    }
                }

                if workflows.is_empty() {
                    div { class: "rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-8 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No workflows yet. Create your first workflow to get started."
                        }
                    }
                } else {
                    div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                        for workflow in workflows.iter().take(6) {
                            WorkflowCard { workflow: workflow.clone() }
                        }
                    }
                }
            }
        }
    }
}
