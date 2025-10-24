use crate::pages::workflow::details::components::*;
use crate::pages::workflow::details::hooks::*;
use dioxus::prelude::*;

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let (execution, loading, error) = use_workflow_execution(id);
    let (mut current_step, total_tasks) = use_task_navigation(execution);
    let current_task_audit = use_filtered_audit(execution, current_step);

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
                    TaskViewer {
                        tasks: exec.tasks.clone(),
                        current_step: current_step(),
                        on_step_click: move |step| current_step.set(step),
                        on_prev: move |_| {
                            let current = current_step();
                            if current > 0 {
                                current_step.set(current - 1);
                            }
                        },
                        on_next: move |_| {
                            let current = current_step();
                            if current < total_tasks().saturating_sub(1) {
                                current_step.set(current + 1);
                            }
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
    }
}
