use dioxus::prelude::*;
use see_core::WorkflowExecution;
use std::time::Duration;

#[derive(Clone)]
pub struct WorkflowExecutionState {
    pub execution: Signal<Option<WorkflowExecution>>,
    pub loading: Signal<bool>,
    pub error: Signal<Option<String>>,
}

pub fn use_workflow_execution(id: String) -> WorkflowExecutionState {
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

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    });

    WorkflowExecutionState {
        execution,
        loading,
        error,
    }
}
