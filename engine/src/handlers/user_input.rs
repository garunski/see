use crate::errors::*;
use crate::types::*;
use async_trait::async_trait;
use tracing::{debug, error, instrument, trace};

pub struct UserInputHandler;

#[async_trait]
impl super::TaskHandler for UserInputHandler {
    #[instrument(skip(self, context), fields(task_id = %task.id, task_name = %task.name))]
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        task: &EngineTask,
    ) -> Result<TaskResult, HandlerError> {
        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "Starting user input task execution"
        );

        let TaskFunction::UserInput {
            prompt,
            input_type,
            required,
            default,
        } = &task.function
        else {
            error!(
                execution_id = %context.execution_id,
                task_id = %task.id,
                "Invalid function type - expected UserInput"
            );
            return Err(HandlerError::InvalidConfiguration(
                "Expected UserInput function".to_string(),
            ));
        };

        trace!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            prompt = %prompt,
            input_type = %input_type,
            required = required,
            "Parsed user input parameters"
        );

        context.log_task(
            task.id.clone(),
            format!("Waiting for user input: {}", prompt),
        );

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "Marking task as WaitingForInput"
        );

        context.update_task_status(task.id.clone(), TaskStatus::WaitingForInput);

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "User input task marked as WaitingForInput"
        );

        context.log(format!("Waiting for user input: {}", prompt));
        context.log_task(
            task.id.clone(),
            format!("Requesting input: {} (type: {})", prompt, input_type),
        );

        let result = TaskResult {
            success: true,
            output: serde_json::json!({
                "waiting_for_input": true,
                "prompt": prompt,
                "input_type": input_type,
                "required": required,
                "default": default.clone(),
            }),
            error: None,
        };

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            success = result.success,
            "User input task execution completed - waiting for input"
        );

        Ok(result)
    }
}
