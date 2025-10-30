use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use s_e_e_persistence::{TaskExecution, TaskExecutionStatus, UserInputRequest};
use tracing::{debug, info};

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

    let execution = store
        .get_workflow_execution(execution_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;

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

    if task.status != TaskExecutionStatus::WaitingForInput {
        return Err(CoreError::Execution(format!(
            "Task {} is not waiting for input (status: {:?})",
            task_id, task.status
        )));
    }

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

    validate_input_value(&input_value, &input_request.input_type)?;

    let mut updated_task = task.clone();
    updated_task.user_input = Some(input_value.clone());

    store
        .save_task_execution(updated_task.clone())
        .await
        .map_err(CoreError::Persistence)?;

    debug!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Task updated with input"
    );

    store
        .fulfill_input_request(&input_request.id, input_value.clone())
        .await
        .map_err(CoreError::Persistence)?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Input provided successfully"
    );

    debug!(
        execution_id = %execution_id,
        "Automatically resuming workflow execution after input provided"
    );

    use crate::api::resume::resume_workflow_execution;

    match resume_workflow_execution(execution_id, None).await {
        Ok(_) => {
            info!(
                execution_id = %execution_id,
                task_id = %task_id,
                "Workflow execution resumed successfully after input"
            );
        }
        Err(e) => {
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

fn validate_input_value(
    value: &str,
    input_type: &s_e_e_persistence::InputType,
) -> Result<(), CoreError> {
    match input_type {
        s_e_e_persistence::InputType::String => {
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
