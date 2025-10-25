use crate::{
    errors::CoreError,
    types::{OutputCallback, WorkflowResult},
};
use tokio::fs;
use tracing::{debug, info, instrument};

use super::custom_engine::{convert_workflow_from_json, CustomWorkflowEngine};

#[instrument(skip(workflow_data, _output_callback), fields(execution_id))]
pub async fn execute_workflow_from_content(
    workflow_data: &str,
    _output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    info!("Using custom workflow engine");
    let store = crate::get_global_store()?;

    // Convert workflow to custom format
    let custom_workflow = convert_workflow_from_json(workflow_data)?;

    // Create custom engine
    let engine = CustomWorkflowEngine::new(store);

    // Execute workflow
    engine.execute_workflow(custom_workflow).await
}

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

#[instrument(skip(output_callback), fields(workflow_id = %workflow_id))]
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    debug!("Fetching workflow definition by ID");
    let store = crate::get_global_store()?;
    let workflow_definition = store.get_workflow_definition(workflow_id).await?;

    debug!(
        workflow_id = %workflow_id,
        workflow_name = %workflow_definition.get_name(),
        "Found workflow definition, executing"
    );

    execute_workflow_from_content(&workflow_definition.content, output_callback).await
}

/// Resume a workflow that is waiting for user input
#[instrument(skip(execution_id), fields(execution_id))]
pub async fn resume_workflow(execution_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;

    info!(
        execution_id = %execution_id,
        "Resuming workflow execution"
    );

    // Load workflow metadata
    let metadata = store.get_workflow_metadata(execution_id).await?;

    // Check if workflow is actually waiting for input
    if metadata.status != crate::persistence::models::WorkflowStatus::WaitingForInput {
        return Err(CoreError::Validation(format!(
            "Workflow {} is not waiting for input (status: {:?})",
            execution_id, metadata.status
        )));
    }

    // Load task executions
    let task_executions = store.get_task_executions(execution_id).await?;

    // Find tasks that are waiting for input
    let waiting_tasks: Vec<_> = task_executions
        .iter()
        .filter(|task| task.status == crate::types::TaskStatus::WaitingForInput)
        .collect();

    if waiting_tasks.is_empty() {
        return Err(CoreError::Validation(format!(
            "No tasks waiting for input in workflow {}",
            execution_id
        )));
    }

    // Resume each waiting task
    for task in waiting_tasks {
        let mut updated_task = task.clone();
        updated_task.status = crate::types::TaskStatus::InProgress;
        updated_task.end_timestamp = String::new(); // Clear end timestamp

        store.save_task_execution(&updated_task).await?;

        info!(
            execution_id = %execution_id,
            task_id = %task.task_id,
            "Resumed task from waiting state"
        );
    }

    // Mark workflow as resumed using new persistence method
    store.mark_workflow_resumed(execution_id).await?;

    info!(
        execution_id = %execution_id,
        "Workflow resumed successfully"
    );

    Ok(())
}

/// Resume a specific task that is waiting for user input
#[instrument(skip(execution_id, task_id), fields(execution_id, task_id))]
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Resuming specific task"
    );

    // Load task execution
    let task_executions = store.get_task_executions(execution_id).await?;
    let task = task_executions
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| {
            CoreError::Validation(format!(
                "Task {} not found in execution {}",
                task_id, execution_id
            ))
        })?;

    // Check if task is waiting for input
    if task.status != crate::types::TaskStatus::WaitingForInput {
        return Err(CoreError::Validation(format!(
            "Task {} is not waiting for input (status: {})",
            task_id, task.status
        )));
    }

    // Update task status
    let mut updated_task = task.clone();
    updated_task.status = crate::types::TaskStatus::InProgress;
    updated_task.end_timestamp = String::new(); // Clear end timestamp

    store.save_task_execution(&updated_task).await?;

    // Check if all tasks are now running or complete
    let all_tasks = store.get_task_executions(execution_id).await?;
    let has_waiting_tasks = all_tasks
        .iter()
        .any(|t| t.status == crate::types::TaskStatus::WaitingForInput);

    info!(
        execution_id = %execution_id,
        task_count = all_tasks.len(),
        has_waiting_tasks = has_waiting_tasks,
        "Checking if workflow should be resumed"
    );

    if !has_waiting_tasks {
        // Mark workflow as resumed using new persistence method
        store.mark_workflow_resumed(execution_id).await?;

        info!(
            execution_id = %execution_id,
            "All tasks resumed, workflow status updated to Running"
        );
    } else {
        info!(
            execution_id = %execution_id,
            "Still has waiting tasks, workflow status remains WaitingForInput"
        );
    }

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Task resumed successfully"
    );

    Ok(())
}

/// Pause a workflow for user input
#[instrument(skip(execution_id, task_id), fields(execution_id, task_id))]
pub async fn pause_workflow(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Pausing workflow for user input"
    );

    // Load task execution
    let task_executions = store.get_task_executions(execution_id).await?;
    let task = task_executions
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| {
            CoreError::Validation(format!(
                "Task {} not found in execution {}",
                task_id, execution_id
            ))
        })?;

    // Update task status to waiting for input
    let mut updated_task = task.clone();
    updated_task.status = crate::types::TaskStatus::WaitingForInput;
    updated_task.end_timestamp = String::new(); // Clear end timestamp

    store.save_task_execution(&updated_task).await?;

    // Mark workflow as paused
    store.mark_workflow_paused(execution_id, task_id).await?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Workflow paused successfully"
    );

    Ok(())
}
