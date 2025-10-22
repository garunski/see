use see_core::{
    errors::CoreError, execute_workflow, execute_workflow_from_content, OutputCallback,
    WorkflowResult,
};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct RunHandles {
    pub receiver: mpsc::Receiver<String>,
}

pub fn create_output_channel() -> (OutputCallback, RunHandles) {
    let (tx, rx) = mpsc::channel(100);
    let tx_clone = tx.clone();
    let output: OutputCallback = Arc::new(move |msg| {
        let _ = tx_clone.try_send(msg);
    });
    (output, RunHandles { receiver: rx })
}

pub async fn run_workflow_from_content(
    content: String,
    output: OutputCallback,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow_from_content(&content, Some(output)).await
}

pub async fn run_workflow(
    file_path: String,
    output: OutputCallback,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow(&file_path, Some(output)).await
}

#[derive(Clone, Debug)]
pub struct WorkflowProgress {
    pub completed: usize,
    pub total: usize,
    pub current_task: Option<String>,
    pub is_complete: bool,
}

pub async fn poll_workflow_progress(execution_id: &str) -> Result<WorkflowProgress, CoreError> {
    let store = see_core::get_global_store()?;
    // Try to get workflow with tasks from new schema
    match store.get_workflow_with_tasks(execution_id).await {
        Ok(execution) => {
            let completed = execution
                .tasks
                .iter()
                .filter(|t| t.status == see_core::TaskStatus::Complete)
                .count();
            let total = execution.tasks.len();

            let current_task = execution
                .tasks
                .iter()
                .find(|t| t.status == see_core::TaskStatus::InProgress)
                .map(|t| t.name.clone());

            Ok(WorkflowProgress {
                completed,
                total,
                current_task,
                is_complete: execution.success,
            })
        }
        Err(e) => Err(e),
    }
}
