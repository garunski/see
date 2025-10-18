use async_trait::async_trait;
use dataflow_rs::{Result, DataflowError};
use dataflow_rs::engine::{
    AsyncFunctionHandler, FunctionConfig,
    message::{Change, Message},
};
use datalogic_rs::DataLogic;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::Arc;
use std::cell::RefCell;

use crate::json_parser;

// Thread-local storage for output callback
thread_local! {
    static OUTPUT_CALLBACK: RefCell<Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>> = RefCell::new(None);
}

/// Set the output callback for CLI command output
pub fn set_output_callback(callback: std::sync::Arc<dyn Fn(String) + Send + Sync>) {
    OUTPUT_CALLBACK.with(|cb| {
        *cb.borrow_mut() = Some(callback);
    });
}

/// Clear the output callback
pub fn clear_output_callback() {
    OUTPUT_CALLBACK.with(|cb| {
        *cb.borrow_mut() = None;
    });
}

/// Helper function to output text (uses callback if set, otherwise println!)
fn output_text(text: &str) {
    OUTPUT_CALLBACK.with(|cb| {
        if let Some(ref callback) = *cb.borrow() {
            callback(text.to_string());
        } else {
            println!("{}", text);
        }
    });
}

/// Custom async function handler for CLI commands
pub struct CliCommandHandler;

#[async_trait]
impl AsyncFunctionHandler for CliCommandHandler {
    async fn execute(
        &self,
        message: &mut Message,
        config: &FunctionConfig,
        _datalogic: Arc<DataLogic>,
    ) -> Result<(usize, Vec<Change>)> {
        // Extract the raw input from config
        let input = match config {
            FunctionConfig::Custom { input, .. } => input,
            _ => {
                return Err(DataflowError::Validation(
                    "Invalid configuration type for CLI command function".to_string(),
                ));
            }
        };

        // Extract command and args from input
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DataflowError::Validation("Missing 'command' field".to_string()))?;
        
        let args = input
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        
        // Extract response type (default to "text" if not specified)
        let response_type = input
            .get("response_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        
        // Format command for easy copy-paste to shell
        let formatted_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };
        output_text(&format!("Executing CLI command: {} (response_type: {})", formatted_command, response_type));
        
        // Execute the command
        let output = Command::new(command)
            .args(&args)
            .output()
            .map_err(|e| DataflowError::function_execution(
                format!("Failed to execute command '{}': {}", command, e),
                None
            ))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if !stdout.is_empty() {
            output_text(&format!("Output: {}", stdout.trim()));
        }
        
        if !stderr.is_empty() {
            output_text(&format!("Error: {}", stderr.trim()));
        }
        
        if !output.status.success() {
            return Err(DataflowError::function_execution(
                format!(
                    "Command '{}' failed with exit code: {:?}\nstderr: {}",
                    command,
                    output.status.code(),
                    stderr
                ),
                None
            ));
        }
        
        // Process JSON based on response type
        let extracted_json = if !stdout.is_empty() {
            match response_type {
                "json" => {
                    // Direct JSON parsing - expect stdout to be valid JSON
                    match serde_json::from_str::<Value>(&stdout) {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            output_text(&format!("Warning: Expected JSON output but parsing failed: {}", e));
                            None
                        }
                    }
                }
                "text" | _ => {
                    // Extract JSON from text using json_parser
                    json_parser::extract_json_from_text(&stdout)
                }
            }
        } else {
            None
        };
        
        // Display extracted JSON if found
        if let Some(ref json_val) = extracted_json {
            output_text("\n🔍 Extracted JSON:");
            output_text(&serde_json::to_string_pretty(json_val).unwrap_or_else(|_| "{}".to_string()));
            
            // Display parsed values
            output_text("\n📋 Parsed Values:");
            display_json_values(json_val, "");
        }
        
        // Create the result value with both raw output and extracted JSON
        let result = json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code().unwrap_or(0),
            "extracted_json": extracted_json
        });
        
        // Store result in message context
        if let Value::Object(ref mut map) = message.context["data"] {
            map.insert("cli_output".to_string(), result.clone());
        }
        
        // Return success with changes
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

/// Helper function to recursively display JSON values
fn display_json_values(value: &Value, prefix: &str) {
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
                        display_json_values(val, &new_prefix);
                    }
                    _ => {
                        output_text(&format!("  - {}: {}", new_prefix, format_value(val)));
                    }
                }
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, idx);
                match val {
                    Value::Object(_) | Value::Array(_) => {
                        display_json_values(val, &new_prefix);
                    }
                    _ => {
                        output_text(&format!("  - {}: {}", new_prefix, format_value(val)));
                    }
                }
            }
        }
        _ => {
            output_text(&format!("  - {}: {}", prefix, format_value(value)));
        }
    }
}

/// Helper function to format JSON values for display
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
    }
}
