use crate::icons::Icon;
use crate::pages::executions::details::components::*;
use crate::pages::executions::details::hooks::*;
use dioxus::prelude::*;
use s_e_e_core::{TaskInfo, WorkflowExecution};

#[component]
fn ResumeButton(execution_id: String, task: TaskInfo) -> Element {
    let task_id = task.id.clone();
    let execution_id_clone = execution_id.clone();

    rsx! {
        div {
            class: "bg-amber-50 border border-amber-200 rounded-lg p-4 mb-4",
            div { class: "flex items-center gap-2 mb-2",
                Icon {
                    name: "pause".to_string(),
                    class: Some("text-amber-600 text-lg".to_string()),
                    size: Some("w-5 h-5".to_string()),
                    variant: Some("outline".to_string()),
                }
                span { class: "text-amber-800 font-medium", "Task Waiting for Input" }
            }
            p { class: "text-amber-700 text-sm mb-3",
                "This task is paused and waiting for user input. Click the button below to resume execution."
            }
            button {
                class: "px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors inline-flex items-center gap-2",
                onclick: move |_| {
                    let execution_id_clone = execution_id_clone.clone();
                    let task_id_clone = task_id.clone();

                    spawn(async move {
                        tracing::info!("Resume button clicked for execution {} task {}", execution_id_clone, task_id_clone);

                        match s_e_e_core::resume_task(&execution_id_clone, &task_id_clone).await {
                            Ok(_) => {
                                tracing::info!("Task resumed successfully");
                                // TODO: Refresh the page or update state in Phase 6
                            }
                            Err(e) => {
                                tracing::error!("Failed to resume task: {}", e);
                                // TODO: Show error message to user in Phase 6
                            }
                        }
                    });
                },
                Icon {
                    name: "play".to_string(),
                    class: Some("w-4 h-4".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
                "Resume Workflow"
            }
        }
    }
}

#[component]
fn TaskFlowVisualizer(
    execution: WorkflowExecution,
    ordered_task_ids: Vec<String>,
    on_task_click: EventHandler<usize>,
) -> Element {
    // Build task map for fast lookup
    let task_map: std::collections::HashMap<_, _> = execution
        .tasks
        .iter()
        .map(|t| (t.id.clone(), t.clone()))
        .collect();

    // Reorder tasks based on snapshot
    let ordered_tasks: Vec<_> = if ordered_task_ids.is_empty() {
        // Fall back to original order if snapshot is empty
        execution.tasks.clone()
    } else {
        ordered_task_ids
            .iter()
            .filter_map(|id| task_map.get(id).cloned())
            .collect()
    };

    rsx! {
        if !ordered_tasks.is_empty() {
            WorkflowFlow {
                tasks: ordered_tasks.iter().map(s_e_e_core::task_execution_to_info).collect(),
                on_task_click
            }
        }
    }
}

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let (execution, loading, error) = use_workflow_execution(id.clone());
    let (current_step, _total_tasks) = use_task_navigation(execution);
    let current_task_audit = use_filtered_audit(execution, current_step);
    let ordered_task_ids = use_task_order_from_snapshot(execution);

    // Panel state
    let mut is_panel_open = use_signal(|| false);
    let mut selected_task_index = use_signal(|| 0);

    // Clone id for use in closures
    let execution_id = id.clone();

    rsx! {
        div { class: "space-y-8",
            ExecutionHeader {}

            if loading() {
                LoadingState {}
            }

            if let Some(err) = error() {
                ErrorState { error: err }
            }

            if let Some(exec) = execution() {
                ExecutionOverview { execution: exec.clone() }

                // Resume button for waiting tasks
                if let Some(task) = exec.tasks.get(selected_task_index()) {
                    if task.status.as_str() == "waiting_for_input" {
                        ResumeButton { execution_id: execution_id.clone(), task: s_e_e_core::task_execution_to_info(task) }
                    }
                }

                // Display tasks in correct execution order
                TaskFlowVisualizer {
                    execution: exec.clone(),
                    ordered_task_ids: ordered_task_ids(),
                    on_task_click: move |task_index: usize| {
                        selected_task_index.set(task_index);
                        is_panel_open.set(true);
                    }
                }

                AuditTrail { audit_entries: current_task_audit() }

                ErrorList { errors: exec.errors.clone() }
            }
        }

        // Task Details Panel - rendered outside main content for true overlay
        TaskDetailsPanel {
            is_open: is_panel_open(),
            current_task: execution().and_then(|exec| exec.tasks.get(selected_task_index()).map(s_e_e_core::task_execution_to_info)),
            current_task_index: selected_task_index(),
            total_tasks: execution().map(|exec| exec.tasks.len()).unwrap_or(0),
            execution: execution(),
            on_close: move |_| is_panel_open.set(false),
            on_previous: move |_| {
                let current = selected_task_index();
                if current > 0 {
                    selected_task_index.set(current - 1);
                }
            },
            on_next: move |_| {
                let current = selected_task_index();
                let total = execution().map(|exec| exec.tasks.len()).unwrap_or(0);
                if current < total.saturating_sub(1) {
                    selected_task_index.set(current + 1);
                }
            }
        }
    }
}
