//! CLI command handler for executing shell commands

use crate::errors::*;
use crate::types::*;
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;
use tracing::{debug, error, instrument, trace, warn};

/// CLI command handler
pub struct CliCommandHandler;

#[async_trait]
impl super::TaskHandler for CliCommandHandler {
    #[instrument(skip(self, context), fields(task_id = %task.id, task_name = %task.name))]
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        task: &EngineTask,
    ) -> Result<TaskResult, HandlerError> {
        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "Starting CLI command execution"
        );

        let TaskFunction::CliCommand { command, args } = &task.function else {
            error!(
                execution_id = %context.execution_id,
                task_id = %task.id,
                "Invalid function type - expected CliCommand"
            );
            return Err(HandlerError::InvalidConfiguration(
                "Expected CliCommand function".to_string(),
            ));
        };

        trace!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            command = %command,
            args = ?args,
            "Parsed CLI command parameters"
        );

        context.log_task(
            task.id.clone(),
            format!("Executing CLI command: {} {:?}", command, args),
        );

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            command = %command,
            args_count = args.len(),
            "Spawning command process"
        );

        // Execute the command
        let output = Command::new(command).args(args).output().map_err(|e| {
            error!(
                execution_id = %context.execution_id,
                task_id = %task.id,
                command = %command,
                error = %e,
                "Failed to spawn command process"
            );
            HandlerError::ExecutionFailed(format!("Failed to execute command: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        trace!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            stdout_length = stdout.len(),
            stderr_length = stderr.len(),
            exit_code = ?output.status.code(),
            "Command execution completed"
        );

        context.log_task(task.id.clone(), format!("Output: {}", stdout));
        if !stderr.is_empty() {
            context.log_task(task.id.clone(), format!("Error: {}", stderr));
        }

        let success = output.status.success();

        if success {
            debug!(
                execution_id = %context.execution_id,
                task_id = %task.id,
                "Command executed successfully"
            );
            context.log_task(task.id.clone(), "Command executed successfully".to_string());
        } else {
            warn!(
                execution_id = %context.execution_id,
                task_id = %task.id,
                exit_code = ?output.status.code(),
                stderr = %stderr,
                "Command failed"
            );
            context.log_task(
                task.id.clone(),
                format!("Command failed with exit code: {:?}", output.status.code()),
            );
        }

        let result = TaskResult {
            success,
            output: Value::String(stdout.to_string()),
            error: if success {
                None
            } else {
                Some(stderr.to_string())
            },
        };

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            success = result.success,
            "CLI command execution completed"
        );

        Ok(result)
    }
}
