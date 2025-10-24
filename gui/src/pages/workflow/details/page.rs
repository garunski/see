use crate::pages::workflow::details::components::*;
use crate::pages::workflow::details::hooks::*;
use dioxus::prelude::*;

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let (execution, loading, error) = use_workflow_execution(id);
    let (current_step, _total_tasks) = use_task_navigation(execution);
    let current_task_audit = use_filtered_audit(execution, current_step);

    // Panel state
    let mut is_panel_open = use_signal(|| false);
    let mut selected_task_index = use_signal(|| 0);

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

                if !exec.tasks.is_empty() {
                    WorkflowFlow {
                        tasks: exec.tasks.clone(),
                        on_task_click: move |task_index| {
                            selected_task_index.set(task_index);
                            is_panel_open.set(true);
                        }
                    }
                }

                AuditTrail { audit_entries: current_task_audit() }

                TaskLogs {
                    current_task: exec.tasks.get(current_step()).cloned(),
                    per_task_logs: exec.per_task_logs.clone()
                }

                ErrorList { errors: exec.errors.clone() }
            }
        }

        // Task Details Panel - rendered outside main content for true overlay
        TaskDetailsPanel {
            is_open: is_panel_open(),
            current_task: execution().and_then(|exec| exec.tasks.get(selected_task_index()).cloned()),
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
