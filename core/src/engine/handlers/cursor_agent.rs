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

pub struct CursorAgentHandler {
    context: Arc<Mutex<ExecutionContext>>,
    persistence: TaskPersistenceHelper,
}

impl CursorAgentHandler {
    pub fn new(context: Arc<Mutex<ExecutionContext>>) -> Self {
        Self {
            context: context.clone(),
            persistence: TaskPersistenceHelper::new(context),
        }
    }

    /// Resolves the prompt either from database or direct input
    async fn resolve_prompt(&self, task_config: &Value) -> Result<String, CoreError> {
        // Check if prompt_id is provided
        if let Some(prompt_id) = task_config.get("prompt_id").and_then(|v| v.as_str()) {
            // Fetch from database
            let store = {
                let ctx = self
                    .context
                    .lock()
                    .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?;
                ctx.get_store()
            };

            if let Some(store) = store {
                match store.get_prompt(prompt_id).await? {
                    Some(prompt) => {
                        debug!(prompt_id = %prompt_id, "Resolved prompt from database");
                        return Ok(prompt.content);
                    }
                    None => {
                        return Err(CoreError::Validation(format!(
                            "Prompt with ID '{}' not found in database",
                            prompt_id
                        )));
                    }
                }
            }
            return Err(CoreError::Validation(
                "No store available to fetch prompt".to_string(),
            ));
        }

        // Fall back to direct prompt field
        task_config
            .get("prompt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                CoreError::Validation("Missing 'prompt' or 'prompt_id' field".to_string())
            })
    }

    /// Builds the cursor-agent command with arguments
    #[allow(clippy::result_large_err)]
    fn build_cursor_agent_command(
        &self,
        prompt: &str,
        task_config: &Value,
    ) -> Result<(String, Vec<String>), CoreError> {
        let mut args = vec!["-p".to_string(), prompt.to_string()];

        // Add output-format based on response_type if specified
        if let Some(response_type) = task_config.get("response_type").and_then(|v| v.as_str()) {
            args.push("--output-format".to_string());
            args.push(response_type.to_string());
        }

        // Add model if specified
        if let Some(model) = task_config.get("model").and_then(|v| v.as_str()) {
            args.push("--model".to_string());
            args.push(model.to_string());
        }

        // Add additional args if specified
        if let Some(additional_args) = task_config.get("args").and_then(|v| v.as_array()) {
            for arg in additional_args {
                if let Some(arg_str) = arg.as_str() {
                    args.push(arg_str.to_string());
                }
            }
        }

        Ok(("cursor-agent".to_string(), args))
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

impl TaskExecutor for CursorAgentHandler {
    #[instrument(skip(self, task_config, logger), fields(task_id))]
    async fn execute(
        &self,
        task_config: &Value,
        logger: &dyn TaskLogger,
    ) -> Result<Value, CoreError> {
        let task_id = task_config
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::Span::current().record("task_id", task_id);
        debug!("Starting cursor-agent task execution");
        logger.start_task(task_id);

        // Resolve prompt from database or direct input
        let prompt = self.resolve_prompt(task_config).await?;
        logger.log(&format!("Using prompt: {}", prompt));

        // Build cursor-agent command
        let (command, args) = self.build_cursor_agent_command(&prompt, task_config)?;

        let response_type = task_config
            .get("response_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        self.persistence
            .save_task_state_async(task_id, TaskStatus::InProgress);

        let formatted_command = format!("{} {}", command, args.join(" "));
        logger.log(&format!(
            "Executing cursor-agent command: {} (response_type: {})",
            formatted_command, response_type
        ));

        debug!(command = %command, args = ?args, "Executing cursor-agent command");
        let output = tokio::process::Command::new(&command)
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                CoreError::CommandExecution(format!(
                    "Failed to execute cursor-agent command '{}': {}",
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
            error!(exit_code = ?output.status.code(), stderr = %stderr, "Cursor-agent command failed");
            logger.end_task(task_id);

            self.persistence
                .save_task_state_async(task_id, TaskStatus::Failed);

            return Err(CoreError::CommandExecution(format!(
                "Cursor-agent command '{}' failed with exit code: {:?}\nstderr: {}",
                command,
                output.status.code(),
                stderr
            )));
        }

        debug!("Cursor-agent command executed successfully");
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
            logger.log("\n🤖 Cursor Agent Response:");
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
            "extracted_json": extracted_json,
            "prompt_used": prompt
        });

        logger.end_task(task_id);

        self.persistence
            .save_task_state_async(task_id, TaskStatus::Complete);

        Ok(result)
    }
}

#[async_trait]
impl AsyncFunctionHandler for CursorAgentHandler {
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
            map.insert("cursor_agent_output".to_string(), result.clone());
        }

        Ok((
            200,
            vec![Change {
                path: Arc::from("cursor_agent_output"),
                old_value: Arc::new(Value::Null),
                new_value: Arc::new(result),
            }],
        ))
    }
}
