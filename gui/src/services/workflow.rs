use see_core::{execute_workflow, OutputCallback, WorkflowResult};
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

pub async fn run_workflow(
    file_path: String,
    output: OutputCallback,
) -> Result<WorkflowResult, Box<dyn std::error::Error>> {
    execute_workflow(&file_path, Some(output)).await
}
