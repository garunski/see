use crate::pages::workflow::upload::components::{StepNavigator, WorkflowProgress};
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use see_core::{AuditStore, TaskStatus, WorkflowExecution};
use std::sync::Arc;
use std::time::Duration;

#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let _state_provider = use_context::<AppStateProvider>();
    let store = use_context::<Option<Arc<see_core::RedbStore>>>();
    let navigator = use_navigator();

    let execution = use_signal(|| None::<WorkflowExecution>);
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);

    // Load workflow execution and set up polling
    use_effect(move || {
        let store = store.clone();
        let mut execution = execution;
        let mut loading = loading;
        let mut error = error;
        let id = id.clone();

        spawn(async move {
            loop {
                if let Some(s) = store.clone() {
                    match s.get_workflow_with_tasks(&id).await {
                        Ok(exec) => {
                            execution.set(Some(exec.clone()));
                            loading.set(false);

                            // Stop polling if workflow is complete or failed
                            if exec.success || !exec.errors.is_empty() {
                                break;
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load workflow: {}", e)));
                            loading.set(false);
                            break;
                        }
                    }
                } else {
                    error.set(Some("Database not available".to_string()));
                    loading.set(false);
                    break;
                }

                // Poll every 2 seconds
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    });

    let mut current_step = use_signal(|| 0);

    // Calculate filtered audit entries for current task
    let current_task_audit = use_memo(move || {
        if let Some(exec) = execution() {
            let current_task_id = exec.tasks.get(current_step()).map(|t| t.id.clone());
            exec.audit_trail
                .iter()
                .filter(|entry| current_task_id.as_ref() == Some(&entry.task_id))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    });

    // Calculate total tasks for navigation
    let total_tasks = use_memo(move || {
        if let Some(exec) = execution() {
            exec.tasks.len()
        } else {
            0
        }
    });

    rsx! {
        div { class: "space-y-8",
            // Header with back button
            div { class: "flex items-center gap-4",
                button {
                    onclick: move |_| navigator.go_back(),
                    class: "flex items-center gap-2 px-3 py-2 text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white transition-colors rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800",
                    // ChevronLeftIcon SVG
                    svg {
                        class: "w-5 h-5",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" }
                    }
                    span { "Back to History" }
                }
                h1 { class: "text-lg font-semibold text-zinc-950 dark:text-white", "Workflow Execution Details" }
            }

            // Loading state
            if loading() {
                div { class: "flex items-center justify-center py-16",
                    div { class: "animate-spin w-8 h-8 border-2 border-zinc-300 border-t-zinc-900 rounded-full dark:border-zinc-600 dark:border-t-zinc-100" }
                }
            }

            // Error state
            if let Some(err) = error() {
                div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6",
                    div { class: "flex items-center gap-3",
                        svg {
                            class: "w-6 h-6 text-red-600 dark:text-red-400",
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                        }
                        div {
                            h3 { class: "text-base font-semibold text-red-800 dark:text-red-200", "Error Loading Workflow" }
                            p { class: "text-red-700 dark:text-red-300 mt-1", "{err}" }
                        }
                    }
                }
            }

            // Main content
            if let Some(exec) = execution() {
                // Metadata Card
                div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{exec.workflow_name}" }
                        div {
                            class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                if exec.success {
                                    "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
                                } else {
                                    "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
                                }
                            ),
                            if exec.success { "Success" } else { "Failed" }
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        div {
                            div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Execution ID" }
                            div { class: "text-zinc-950 dark:text-white font-mono text-sm", "{exec.id}" }
                        }
                        div {
                            div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Executed At" }
                            div { class: "text-zinc-950 dark:text-white", "{exec.timestamp}" }
                        }
                        div {
                            div { class: "text-sm text-zinc-500 dark:text-zinc-400", "Tasks" }
                            div { class: "text-zinc-950 dark:text-white", "{exec.tasks.len()} total" }
                        }
                    }
                }

                // Current Task Details
                if !exec.tasks.is_empty() {
                    div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                        h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Current Task" }

                        // Task Progress Overview
                        WorkflowProgress {
                            tasks: exec.tasks.clone(),
                            current_step: current_step(),
                            on_step_click: move |step| current_step.set(step)
                        }

                        // Current Task Information
                        if let Some(task) = exec.tasks.get(current_step()) {
                            div { class: "mt-6 p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg",
                                div { class: "flex items-center justify-between mb-3",
                                    h4 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{task.name}" }
                                    div {
                                        class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                            match task.status {
                                                TaskStatus::Complete => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
                                                TaskStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                                                TaskStatus::InProgress => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                                TaskStatus::Pending => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
                                            }
                                        ),
                                        match task.status {
                                            TaskStatus::Complete => "Complete",
                                            TaskStatus::Failed => "Failed",
                                            TaskStatus::InProgress => "In Progress",
                                            TaskStatus::Pending => "Pending",
                                        }
                                    }
                                }
                                div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                                    "Task ID: {task.id}"
                                }
                            }
                        }

                        // Step Navigator
                        div { class: "mt-4",
                            StepNavigator {
                                current_step: current_step(),
                                total_steps: exec.tasks.len(),
                                task_name: exec.tasks.get(current_step()).map(|t| t.name.clone()).unwrap_or_default(),
                                task_status: exec.tasks.get(current_step()).map(|t| t.status.clone()).unwrap_or(TaskStatus::Pending),
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
                    }
                }

                // Current Task Audit Trail
                if !current_task_audit().is_empty() {
                    div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                        h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Current Task Audit Trail" }
                        div { class: "space-y-3",
                            for entry in current_task_audit().iter() {
                                div { class: "flex items-start gap-3 p-3 bg-zinc-50 dark:bg-zinc-800 rounded-lg",
                                    div { class: "w-2 h-2 bg-blue-500 rounded-full mt-2 flex-shrink-0" }
                                    div { class: "flex-1 min-w-0",
                                        div { class: "text-sm text-zinc-950 dark:text-white font-medium", "Task: {entry.task_id}" }
                                        div { class: "text-xs text-zinc-500 dark:text-zinc-400 mt-1", "{entry.timestamp} - Status: {entry.status} - Changes: {entry.changes_count}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Current Task Logs
                if let Some(current_task) = exec.tasks.get(current_step()) {
                    if let Some(logs) = exec.per_task_logs.get(&current_task.id) {
                        if !logs.is_empty() {
                            div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                                h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Current Task Logs" }
                                div { class: "space-y-2 max-h-64 overflow-y-auto",
                                    for log in logs.iter() {
                                        div { class: "text-sm text-zinc-700 dark:text-zinc-300 font-mono bg-zinc-100 dark:bg-zinc-800 p-2 rounded", "{log}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Errors Section
                if !exec.errors.is_empty() {
                    div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                        h3 { class: "text-base font-semibold text-red-800 dark:text-red-200 mb-4", "Errors" }
                        div { class: "space-y-3",
                            for error in exec.errors.iter() {
                                div { class: "p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg",
                                    div { class: "flex items-start gap-3",
                                        svg {
                                            class: "w-5 h-5 text-red-600 dark:text-red-400 mt-0.5 flex-shrink-0",
                                            view_box: "0 0 20 20",
                                            fill: "currentColor",
                                            path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                                        }
                                        div { class: "text-sm text-red-800 dark:text-red-200", "{error}" }
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
