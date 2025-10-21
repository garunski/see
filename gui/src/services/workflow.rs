use see_core::{
    errors::CoreError, execute_workflow, execute_workflow_from_content, AuditStore, OutputCallback,
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
    store: Option<Arc<dyn AuditStore>>,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow_from_content(&content, Some(output), store).await
}

pub async fn run_workflow(
    file_path: String,
    output: OutputCallback,
    store: Option<Arc<dyn AuditStore>>,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow(&file_path, Some(output), store).await
}
