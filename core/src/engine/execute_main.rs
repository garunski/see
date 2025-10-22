use crate::errors::CoreError;
use crate::types::{OutputCallback, WorkflowResult};
use tokio::fs;
use tracing::{debug, instrument};

use super::execute::WorkflowExecutor;

/// Execute workflow from JSON content using modern executor
#[instrument(skip(workflow_data, output_callback), fields(execution_id))]
pub async fn execute_workflow_from_content(
    workflow_data: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    let executor = WorkflowExecutor::new();
    executor
        .execute_from_content(workflow_data, output_callback)
        .await
}

/// Execute workflow from file using modern executor
#[instrument(skip(output_callback), fields(workflow_file = %workflow_file))]
pub async fn execute_workflow(
    workflow_file: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    debug!("Reading workflow file");
    let workflow_data = fs::read_to_string(workflow_file).await.map_err(|e| {
        CoreError::WorkflowExecution(format!(
            "Failed to read workflow file '{}': {}",
            workflow_file, e
        ))
    })?;

    execute_workflow_from_content(&workflow_data, output_callback).await
}
