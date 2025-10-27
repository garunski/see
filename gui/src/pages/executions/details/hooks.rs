use dioxus::prelude::*;
use s_e_e_core::WorkflowExecution;
use serde_json::Value;
use std::time::Duration;

const POLLING_INTERVAL_SECS: u64 = 2;

/// Hook for managing workflow execution data with polling
pub fn use_workflow_execution(
    id: String,
) -> (
    Signal<Option<WorkflowExecution>>,
    Signal<bool>,
    Signal<Option<String>>,
) {
    let execution = use_signal(|| None::<WorkflowExecution>);
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);

    use_effect(move || {
        let mut execution = execution;
        let mut loading = loading;
        let mut error = error;
        let id = id.clone();

        spawn(async move {
            loop {
                match s_e_e_core::get_global_store() {
                    Ok(store) => match store.get_workflow_with_tasks(&id).await {
                        Ok(exec) => {
                            execution.set(Some(exec.clone()));
                            loading.set(false);

                            if exec.success.unwrap_or(false) || !exec.errors.is_empty() {
                                break;
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load workflow: {}", e)));
                            loading.set(false);
                            break;
                        }
                    },
                    Err(e) => {
                        error.set(Some(format!("Database not available: {}", e)));
                        loading.set(false);
                        break;
                    }
                }

                tokio::time::sleep(Duration::from_secs(POLLING_INTERVAL_SECS)).await;
            }
        });
    });

    (execution, loading, error)
}

/// Hook for managing task navigation state
pub fn use_task_navigation(
    execution: Signal<Option<WorkflowExecution>>,
) -> (Signal<usize>, Signal<usize>) {
    let current_step = use_signal(|| 0);

    let mut total_tasks = use_signal(|| 0);

    use_effect(move || {
        if let Some(exec) = execution() {
            total_tasks.set(exec.tasks.len());
        } else {
            total_tasks.set(0);
        }
    });

    (current_step, total_tasks)
}

/// Hook for filtering audit trail by current task
pub fn use_filtered_audit(
    execution: Signal<Option<WorkflowExecution>>,
    current_step: Signal<usize>,
) -> Signal<Vec<s_e_e_core::AuditEntry>> {
    let mut audit_entries = use_signal(Vec::<s_e_e_core::AuditEntry>::new);

    use_effect(move || {
        if let Some(exec) = execution() {
            let current_task_id = exec.tasks.get(current_step()).map(|t| t.id.clone());
            let filtered: Vec<_> = exec
                .audit_trail
                .iter()
                .filter(|entry| current_task_id.as_ref() == Some(&entry.task_id))
                .map(s_e_e_core::audit_event_to_entry)
                .collect();
            audit_entries.set(filtered);
        } else {
            audit_entries.set(Vec::new());
        }
    });

    audit_entries
}

/// Extract task IDs in execution order from workflow snapshot
pub fn use_task_order_from_snapshot(
    execution: Signal<Option<WorkflowExecution>>,
) -> Memo<Vec<String>> {
    use_memo(move || {
        execution()
            .map(|exec| extract_task_ids_recursive(&exec.workflow_snapshot))
            .unwrap_or_default()
    })
}

/// Recursively extract task IDs from workflow JSON preserving execution order
/// Uses breadth-first traversal to maintain level-by-level order
fn extract_task_ids_recursive(value: &Value) -> Vec<String> {
    let mut task_ids = Vec::new();
    let mut queue: Vec<&Value> = Vec::new();

    // Initialize queue with root tasks
    if let Some(tasks) = value.get("tasks").and_then(|v| v.as_array()) {
        for task in tasks {
            queue.push(task);
        }
    }

    // Process tasks breadth-first
    while !queue.is_empty() {
        let current = queue.remove(0);

        if let Some(task_id) = current.get("id").and_then(|v| v.as_str()) {
            task_ids.push(task_id.to_string());

            // Add next_tasks to queue for processing later
            if let Some(next_tasks) = current.get("next_tasks").and_then(|v| v.as_array()) {
                for next_task in next_tasks {
                    queue.push(next_task);
                }
            }
        }
    }

    task_ids
}

/// Hook for fetching UserInputRequest for a specific task
pub fn use_input_request(
    task_id: String,
    execution_id: String,
) -> Signal<Option<s_e_e_core::UserInputRequest>> {
    let input_request = use_signal(|| None::<s_e_e_core::UserInputRequest>);

    use_effect(move || {
        let task_id = task_id.clone();
        let execution_id = execution_id.clone();
        let mut input_request = input_request;

        spawn(async move {
            match s_e_e_core::get_pending_inputs(&execution_id).await {
                Ok(requests) => {
                    // Find the request matching this task_id
                    let matching_request = requests
                        .iter()
                        .find(|req| req.task_execution_id == task_id)
                        .cloned();
                    input_request.set(matching_request);
                }
                Err(_) => {
                    // No input request found or error loading
                    input_request.set(None);
                }
            }
        });
    });

    input_request
}
