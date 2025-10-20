use crate::execution::context::ExecutionContext;
use async_trait::async_trait;
use dataflow_rs::engine::{
    message::{Change, Message},
    AsyncFunctionHandler, FunctionConfig,
};
use dataflow_rs::{DataflowError, Result};
use datalogic_rs::DataLogic;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub struct CliCommandHandler {
    context: Arc<Mutex<ExecutionContext>>,
}

impl CliCommandHandler {
    pub fn new(context: Arc<Mutex<ExecutionContext>>) -> Self {
        Self { context }
    }

    fn log(&self, msg: &str) {
        if let Ok(mut ctx) = self.context.lock() {
            ctx.log(msg);
        }
    }

    fn display_json_values(&self, value: &Value, prefix: &str) {
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
                            self.display_json_values(val, &new_prefix)
                        }
                        _ => self.log(&format!("  - {}: {}", new_prefix, Self::format_value(val))),
                    }
                }
            }
            Value::Array(arr) => {
                for (idx, val) in arr.iter().enumerate() {
                    let new_prefix = format!("{}[{}]", prefix, idx);
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            self.display_json_values(val, &new_prefix)
                        }
                        _ => self.log(&format!("  - {}: {}", new_prefix, Self::format_value(val))),
                    }
                }
            }
            _ => self.log(&format!("  - {}: {}", prefix, Self::format_value(value))),
        }
    }

    fn format_value(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
        }
    }
}

#[async_trait]
impl AsyncFunctionHandler for CliCommandHandler {
    async fn execute(
        &self,
        message: &mut Message,
        config: &FunctionConfig,
        _datalogic: Arc<DataLogic>,
    ) -> Result<(usize, Vec<Change>)> {
        let input = match config {
            FunctionConfig::Custom { input, .. } => input,
            _ => {
                return Err(DataflowError::Validation(
                    "Invalid configuration".to_string(),
                ))
            }
        };

        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DataflowError::Validation("Missing 'command' field".to_string()))?;

        let args: Vec<String> = input
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let response_type = input
            .get("response_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let task_id = input
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        self.log(&format!("[TASK_START:{}]", task_id));

        let formatted_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };
        self.log(&format!(
            "Executing CLI command: {} (response_type: {})",
            formatted_command, response_type
        ));

        let output = tokio::process::Command::new(command)
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                DataflowError::function_execution(
                    format!("Failed to execute command '{}': {}", command, e),
                    None,
                )
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stdout.is_empty() {
            self.log(&format!("Output: {}", stdout.trim()));
        }
        if !stderr.is_empty() {
            self.log(&format!("Error: {}", stderr.trim()));
        }

        if !output.status.success() {
            self.log(&format!("[TASK_END:{}]", task_id));
            return Err(DataflowError::function_execution(
                format!(
                    "Command '{}' failed with exit code: {:?}\nstderr: {}",
                    command,
                    output.status.code(),
                    stderr
                ),
                None,
            ));
        }

        let extracted_json = if !stdout.is_empty() {
            match response_type {
                "json" => serde_json::from_str::<Value>(&stdout).ok(),
                _ => crate::json_parser::extract_json_from_text(&stdout),
            }
        } else {
            None
        };

        if let Some(ref json_val) = extracted_json {
            self.log("\n🔍 Extracted JSON:");
            self.log(&serde_json::to_string_pretty(json_val).unwrap_or_else(|_| "{}".to_string()));
            self.log("\n📋 Parsed Values:");
            self.display_json_values(json_val, "");
        }

        let result = json!({ "stdout": stdout, "stderr": stderr, "exit_code": output.status.code().unwrap_or(0), "extracted_json": extracted_json });

        if let Some(Value::Object(ref mut map)) = message.context.get_mut("data") {
            map.insert("cli_output".to_string(), result.clone());
        }

        self.log(&format!("[TASK_END:{}]", task_id));

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
