use dioxus::prelude::*;
use see_core::WorkflowExecution;
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
                match see_core::get_global_store() {
                    Ok(store) => match store.get_workflow_with_tasks(&id).await {
                        Ok(exec) => {
                            execution.set(Some(exec.clone()));
                            loading.set(false);

                            if exec.success || !exec.errors.is_empty() {
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
) -> Signal<Vec<see_core::AuditEntry>> {
    let mut audit_entries = use_signal(Vec::<see_core::AuditEntry>::new);

    use_effect(move || {
        if let Some(exec) = execution() {
            let current_task_id = exec.tasks.get(current_step()).map(|t| t.id.clone());
            let filtered: Vec<_> = exec
                .audit_trail
                .iter()
                .filter(|entry| current_task_id.as_ref() == Some(&entry.task_id))
                .cloned()
                .collect();
            audit_entries.set(filtered);
        } else {
            audit_entries.set(Vec::new());
        }
    });

    audit_entries
}
