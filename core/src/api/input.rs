//! Input management API
//!
//! This file contains ONLY input management operations following Single Responsibility Principle.

use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use s_e_e_persistence::{TaskExecution, TaskExecutionStatus, UserInputRequest};
use tracing::{debug, info};

/// Provide user input for a waiting task
///
/// # Arguments
/// * `execution_id` - The workflow execution ID
/// * `task_id` - The task execution ID
/// * `input_value` - The user-provided input value
///
/// # Returns
/// * `Ok(())` if input was provided successfully
/// * `Err(CoreError)` if input could not be provided
pub async fn provide_user_input(
    execution_id: &str,
    task_id: &str,
    input_value: String,
) -> Result<(), CoreError> {
    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Providing user input for task"
    );

    let store = get_global_store()?;

    // Step 1: Get the workflow execution
    let execution = store
        .get_workflow_execution(execution_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;

    // Step 2: Find the task
    let task = execution
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| CoreError::TaskNotFound(task_id.to_string()))?;

    debug!(
        execution_id = %execution_id,
        task_id = %task_id,
        status = ?task.status,
        "Found task for input"
    );

    // Step 3: Validate task status
    if task.status != TaskExecutionStatus::WaitingForInput {
        return Err(CoreError::Execution(format!(
            "Task {} is not waiting for input (status: {:?})",
            task_id, task.status
        )));
    }

    // Step 4: Get input request
    let input_request = store
        .get_input_request_by_task(task_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::Execution("Input request not found".to_string()))?;

    debug!(
        execution_id = %execution_id,
        task_id = %task_id,
        input_type = ?input_request.input_type,
        "Found input request"
    );

    // Step 5: Validate input value against input type
    validate_input_value(&input_value, &input_request.input_type)?;

    // Step 6: Update task with input
    let mut updated_task = task.clone();
    updated_task.user_input = Some(input_value.clone());
    // Keep status as WaitingForInput until we mark it complete after resume

    store
        .save_task_execution(updated_task.clone())
        .await
        .map_err(CoreError::Persistence)?;

    debug!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Task updated with input"
    );

    // Step 7: Mark input request as fulfilled
    store
        .fulfill_input_request(&input_request.id, input_value.clone())
        .await
        .map_err(CoreError::Persistence)?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Input provided successfully"
    );

    // Step 8: Resume workflow execution automatically
    // The task status remains WaitingForInput until resume marks it Complete
    debug!(
        execution_id = %execution_id,
        "Automatically resuming workflow execution after input provided"
    );

    // Import resume_workflow_execution
    use crate::api::resume::resume_workflow_execution;

    // Resume the workflow - this will continue execution from where it left off
    // The resume function will mark tasks with user_input as complete
    match resume_workflow_execution(execution_id, None).await {
        Ok(_) => {
            info!(
                execution_id = %execution_id,
                task_id = %task_id,
                "Workflow execution resumed successfully after input"
            );
        }
        Err(e) => {
            // Log error but don't fail the input provision
            // The user input was successfully stored, workflow resume can be retried
            // For now, manually mark the task as Complete if resume fails
            debug!(
                execution_id = %execution_id,
                task_id = %task_id,
                error = %e,
                "Failed to resume workflow execution, marking task as complete manually"
            );
            updated_task.status = TaskExecutionStatus::Complete;
            updated_task.completed_at = Some(chrono::Utc::now());
            let _ = store.save_task_execution(updated_task).await;
        }
    }

    Ok(())
}

/// Get pending input requests for a workflow
///
/// # Arguments
/// * `workflow_id` - The workflow execution ID
///
/// # Returns
/// * `Ok(Vec<UserInputRequest>)` - List of pending input requests
/// * `Err(CoreError)` if requests could not be retrieved
pub async fn get_pending_inputs(workflow_id: &str) -> Result<Vec<UserInputRequest>, CoreError> {
    debug!(
        workflow_id = %workflow_id,
        "Fetching pending inputs for workflow"
    );

    let store = get_global_store()?;

    let requests = store
        .get_pending_inputs_for_workflow(workflow_id)
        .await
        .map_err(CoreError::Persistence)?;

    info!(
        workflow_id = %workflow_id,
        count = requests.len(),
        "Found pending inputs"
    );

    Ok(requests)
}

/// Get all tasks waiting for input in a workflow
///
/// # Arguments
/// * `workflow_id` - The workflow execution ID
///
/// # Returns
/// * `Ok(Vec<TaskExecution>)` - List of tasks waiting for input
/// * `Err(CoreError)` if tasks could not be retrieved
pub async fn get_tasks_waiting_for_input(
    workflow_id: &str,
) -> Result<Vec<TaskExecution>, CoreError> {
    debug!(
        workflow_id = %workflow_id,
        "Fetching tasks waiting for input"
    );

    let store = get_global_store()?;

    let tasks = store
        .get_tasks_waiting_for_input_in_workflow(workflow_id)
        .await
        .map_err(CoreError::Persistence)?;

    info!(
        workflow_id = %workflow_id,
        count = tasks.len(),
        "Found tasks waiting for input"
    );

    Ok(tasks)
}

/// Validate input value against input type
///
/// # Arguments
/// * `value` - The input value to validate
/// * `input_type` - The expected input type
///
/// # Returns
/// * `Ok(())` if value is valid
/// * `Err(CoreError)` if value is invalid
fn validate_input_value(
    value: &str,
    input_type: &s_e_e_persistence::InputType,
) -> Result<(), CoreError> {
    match input_type {
        s_e_e_persistence::InputType::String => {
            // Any non-empty string is valid
            if value.is_empty() {
                return Err(CoreError::Execution(
                    "Input value cannot be empty".to_string(),
                ));
            }
            Ok(())
        }
        s_e_e_persistence::InputType::Number => value
            .parse::<f64>()
            .map_err(|e| CoreError::Execution(format!("Invalid number format: {}", e)))
            .map(|_| ()),
        s_e_e_persistence::InputType::Boolean => match value.to_lowercase().as_str() {
            "true" | "false" | "1" | "0" | "yes" | "no" => Ok(()),
            _ => Err(CoreError::Execution(format!(
                "Invalid boolean format: expected true/false/1/0/yes/no, got {}",
                value
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input_value_string() {
        assert!(validate_input_value("hello", &s_e_e_persistence::InputType::String).is_ok());
        assert!(validate_input_value("", &s_e_e_persistence::InputType::String).is_err());
    }

    #[test]
    fn test_validate_input_value_number() {
        assert!(validate_input_value("123", &s_e_e_persistence::InputType::Number).is_ok());
        assert!(validate_input_value("3.14", &s_e_e_persistence::InputType::Number).is_ok());
        assert!(validate_input_value("abc", &s_e_e_persistence::InputType::Number).is_err());
    }

    #[test]
    fn test_validate_input_value_boolean() {
        assert!(validate_input_value("true", &s_e_e_persistence::InputType::Boolean).is_ok());
        assert!(validate_input_value("false", &s_e_e_persistence::InputType::Boolean).is_ok());
        assert!(validate_input_value("1", &s_e_e_persistence::InputType::Boolean).is_ok());
        assert!(validate_input_value("0", &s_e_e_persistence::InputType::Boolean).is_ok());
        assert!(validate_input_value("yes", &s_e_e_persistence::InputType::Boolean).is_ok());
        assert!(validate_input_value("no", &s_e_e_persistence::InputType::Boolean).is_ok());
        assert!(validate_input_value("maybe", &s_e_e_persistence::InputType::Boolean).is_err());
    }
}
