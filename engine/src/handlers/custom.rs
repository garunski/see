//! Custom handler for unknown or user-defined function types

use crate::types::*;
use crate::errors::*;
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, trace, error, instrument};

/// Custom handler for unknown function types
pub struct CustomHandler;

#[async_trait]
impl super::TaskHandler for CustomHandler {
    #[instrument(skip(self, context), fields(task_id = %task.id, task_name = %task.name))]
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        task: &EngineTask,
    ) -> Result<TaskResult, HandlerError> {
        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "Starting custom function execution"
        );

        let TaskFunction::Custom { name, input } = &task.function else {
            error!(
                execution_id = %context.execution_id,
                task_id = %task.id,
                "Invalid function type - expected Custom"
            );
            return Err(HandlerError::InvalidConfiguration(
                "Expected Custom function".to_string()
            ));
        };

        trace!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            function_name = %name,
            input_type = ?input,
            "Parsed custom function parameters"
        );

        context.log_task(task.id.clone(), format!("Executing custom function: {}", name));
        context.log_task(task.id.clone(), format!("Input: {}", input));

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            function_name = %name,
            input_preview = %input.to_string().chars().take(100).collect::<String>(),
            "Processing custom function"
        );

        // For now, just echo the input
        // In a real implementation, this would call the actual custom function
        trace!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "Simulating custom function execution"
        );

        let output = format!("Custom function '{}' executed with input: {}", name, input);
        context.log_task(task.id.clone(), format!("Output: {}", output));

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            function_name = %name,
            output_length = output.len(),
            "Generated custom function output"
        );

        let result = TaskResult {
            success: true,
            output: Value::String(output),
            error: None,
        };

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            function_name = %name,
            success = result.success,
            "Custom function execution completed"
        );

        Ok(result)
    }
}
