use crate::components::ui::{Icon, IconName, Loading};
use dioxus::prelude::*;

mod components;
mod hooks;
mod utils;

use components::{AuditTrail, ErrorList, ExecutionHeader, ExecutionSummary, TaskLogs, TaskViewer};
use hooks::use_workflow_execution;
use utils::{get_current_task_audit, get_current_task_logs};

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let state = use_workflow_execution(id);
    let mut current_step = use_signal(|| 0);

    let current_task_id = use_memo(move || {
        if let Some(exec) = state.execution.read().as_ref() {
            exec.tasks.get(current_step()).map(|t| t.id.clone())
        } else {
            None
        }
    });

    let current_task_audit = use_memo(move || {
        if let Some(exec) = state.execution.read().as_ref() {
            get_current_task_audit(exec, current_task_id.as_ref().as_deref())
        } else {
            Vec::new()
        }
    });

    let current_task_logs = use_memo(move || {
        if let Some(exec) = state.execution.read().as_ref() {
            get_current_task_logs(exec, current_task_id.as_ref().as_deref())
        } else {
            Vec::new()
        }
    });

    let total_tasks = use_memo(move || {
        if let Some(exec) = state.execution.read().as_ref() {
            exec.tasks.len()
        } else {
            0
        }
    });

    rsx! {
        div { class: "space-y-8",
            ExecutionHeader {}

            if *state.loading.read() {
                Loading { class: None }
            }

            if let Some(err) = state.error.read().as_ref() {
                div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6",
                    div { class: "flex items-center gap-3",
                        Icon {
                            name: IconName::Error,
                            class: Some("w-6 h-6 text-red-600 dark:text-red-400".to_string())
                        }
                        div {
                            h3 { class: "text-base font-semibold text-red-800 dark:text-red-200", "Error Loading Workflow" }
                            p { class: "text-red-700 dark:text-red-300 mt-1", "{err}" }
                        }
                    }
                }
            }

            if let Some(exec) = state.execution.read().as_ref() {
                ExecutionSummary { execution: exec.clone() }

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

                AuditTrail { entries: current_task_audit() }
                TaskLogs { logs: current_task_logs() }
                ErrorList { errors: exec.errors.clone() }
            }
        }
    }
}
