use crate::errors::CoreError;
use crate::execution::context::ExecutionContext;
use crate::task_executor::{TaskExecutor, TaskLogger, TaskPersistenceHelper};
use crate::types::TaskStatus;
use async_trait::async_trait;
use dataflow_rs::engine::{
    message::{Change, Message},
    AsyncFunctionHandler, FunctionConfig,
};
use dataflow_rs::DataflowError;
use datalogic_rs::DataLogic;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, instrument};

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

        self.persistence
            .save_task_state_async(task_id, TaskStatus::Complete);

        Ok(result)
    }
}

#[async_trait]
impl AsyncFunctionHandler for CliCommandHandler {
    async fn execute(
        &self,
        message: &mut Message,
        config: &FunctionConfig,
        _datalogic: Arc<DataLogic>,
    ) -> dataflow_rs::Result<(usize, Vec<Change>)> {
        let input = match config {
            FunctionConfig::Custom { input, .. } => input,
            _ => {
                return Err(DataflowError::Validation(
                    "Invalid configuration".to_string(),
                ))
            }
        };

        let logger = crate::task_executor::ContextTaskLogger::new(self.context.clone());

        let result = TaskExecutor::execute(self, input, &logger)
            .await
            .map_err(|e| DataflowError::function_execution(e.to_string(), None))?;

        if let Some(Value::Object(ref mut map)) = message.context.get_mut("data") {
            map.insert("cli_output".to_string(), result.clone());
        }

        Ok((
            200,
            vec![Change {
                path: Arc::from("cli_output"),
                old_value: Arc::new(Value::Null),
                new_value: Arc::new(result),
            }],
        ))
    }
}
