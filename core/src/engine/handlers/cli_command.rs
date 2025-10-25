use crate::errors::CoreError;
use crate::execution::context::{ExecutionContext, ExecutionContextSafe};
use crate::task_executor::{TaskExecutor, TaskLogger, TaskPersistenceHelper};
use crate::types::TaskStatus;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, instrument};

use crate::engine::custom_engine::{CustomTask, TaskFunction, TaskHandler, TaskResult};

pub struct CliCommandHandler {
    context: Arc<Mutex<ExecutionContext>>,
    persistence: TaskPersistenceHelper,
}

impl CliCommandHandler {
    pub fn new(context: Arc<Mutex<ExecutionContext>>) -> Self {
        Self {
            context: context.clone(),
            persistence: TaskPersistenceHelper::new(context),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn display_json_values(&self, value: &Value, prefix: &str, logger: &dyn TaskLogger) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            self.display_json_values(val, &new_prefix, logger)
                        }
                        _ => {
                            logger.log(&format!("  - {}: {}", new_prefix, Self::format_value(val)))
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (idx, val) in arr.iter().enumerate() {
                    let new_prefix = format!("{}[{}]", prefix, idx);
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            self.display_json_values(val, &new_prefix, logger)
                        }
                        _ => {
                            logger.log(&format!("  - {}: {}", new_prefix, Self::format_value(val)))
                        }
                    }
                }
            }
            _ => logger.log(&format!("  - {}: {}", prefix, Self::format_value(value))),
        }
    }

    fn format_value(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(value).unwrap_or_else(|e| {
                eprintln!("Failed to serialize value: {}", e);
                "{}".to_string()
            }),
        }
    }
}

impl TaskExecutor for CliCommandHandler {
    #[instrument(skip(self, task_config, logger), fields(task_id))]
    async fn execute(
        &self,
        task_config: &Value,
        logger: &dyn TaskLogger,
    ) -> Result<Value, CoreError> {
        let command = task_config
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::Validation("Missing 'command' field".to_string()))?;

        let args: Vec<String> = task_config
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let response_type = task_config
            .get("response_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let task_id = task_config
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::Span::current().record("task_id", task_id);
        debug!("Starting task execution");
        logger.start_task(task_id);

        self.persistence
            .save_task_state_async(task_id, TaskStatus::InProgress);

        let formatted_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        logger.log(&format!(
            "Executing CLI command: {} (response_type: {})",
            formatted_command, response_type
        ));

        debug!(command = %command, args = ?args, "Executing CLI command");
        let output = tokio::process::Command::new(command)
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                CoreError::CommandExecution(format!(
                    "Failed to execute command '{}': {}",
                    command, e
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stdout.is_empty() {
            logger.log(&format!("Output: {}", stdout.trim()));
        }
        if !stderr.is_empty() {
            logger.log(&format!("Error: {}", stderr.trim()));
        }

        if !output.status.success() {
            error!(exit_code = ?output.status.code(), stderr = %stderr, "Command failed");
            logger.end_task(task_id);

            self.persistence
                .save_task_state_async(task_id, TaskStatus::Failed);

            return Err(CoreError::CommandExecution(format!(
                "Command '{}' failed with exit code: {:?}\nstderr: {}",
                command,
                output.status.code(),
                stderr
            )));
        }

        debug!("Command executed successfully");
        let extracted_json = if !stdout.is_empty() {
            match response_type {
                "json" => Some(serde_json::from_str::<Value>(&stdout).map_err(|e| {
                    CoreError::Serialization(format!("Failed to parse JSON output: {}", e))
                })?),
                _ => crate::json_parser::extract_json_from_text(&stdout),
            }
        } else {
            None
        };

        if let Some(ref json_val) = extracted_json {
            logger.log("\n🔍 Extracted JSON:");
            let json_str = serde_json::to_string_pretty(json_val).map_err(|e| {
                CoreError::Serialization(format!("Failed to serialize JSON: {}", e))
            })?;
            logger.log(&json_str);
            logger.log("\n📋 Parsed Values:");
            self.display_json_values(json_val, "", logger);
        }

        let result = json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code().unwrap_or(0),
            "extracted_json": extracted_json
        });

        logger.end_task(task_id);

        // Check if this task should pause for user input
        debug!(task_id = %task_id, task_config = ?task_config, "Checking for pause_for_input configuration");
        if let Some(pause_config) = task_config.get("pause_for_input") {
            debug!(task_id = %task_id, pause_config = ?pause_config, "Found pause_for_input configuration");
            if let Some(prompt) = pause_config.get("prompt").and_then(|v| v.as_str()) {
                info!(
                    task_id = %task_id,
                    prompt = %prompt,
                    "Pausing workflow for user input"
                );

                // Pause the workflow
                self.context.safe_pause_for_input(task_id, prompt)?;

                // Save task as waiting for input
                self.persistence
                    .save_task_state_async(task_id, TaskStatus::WaitingForInput);

                return Ok(result);
            }
        }

        self.persistence
            .save_task_state_async(task_id, TaskStatus::Complete);

        Ok(result)
    }
}

// Custom engine TaskHandler implementation
#[async_trait]
impl TaskHandler for CliCommandHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &CustomTask,
    ) -> Result<TaskResult, CoreError> {
        // Extract task configuration from CustomTask
        let task_config = match &task.function {
            TaskFunction::CliCommand { command, args } => {
                json!({
                    "command": command,
                    "args": args,
                    "task_id": task.id
                })
            }
            _ => {
                return Err(CoreError::Validation(format!(
                    "Expected CliCommand function, got: {:?}",
                    task.function
                )));
            }
        };

        // Use existing TaskExecutor logic
        let logger = crate::task_executor::ContextTaskLogger::new(context.clone());
        let result = TaskExecutor::execute(self, &task_config, &logger).await?;

        Ok(TaskResult {
            success: true,
            output: result,
            error: None,
        })
    }
}
