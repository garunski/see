use async_trait::async_trait;
use dataflow_rs::{Engine, Workflow, Result, DataflowError};
use dataflow_rs::engine::{
    AsyncFunctionHandler, FunctionConfig,
    message::{Change, Message},
};
use datalogic_rs::DataLogic;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use std::fs;
use std::sync::Arc;

mod json_parser;

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
        
        println!("Executing CLI command: {} {:?}", command, args);
        
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
            println!("Output: {}", stdout.trim());
        }
        
        if !stderr.is_empty() {
            println!("Error: {}", stderr.trim());
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
        
        // Try to extract JSON from stdout using json_parser
        let extracted_json = if !stdout.is_empty() {
            json_parser::extract_json_from_text(&stdout)
        } else {
            None
        };
        
        // Display extracted JSON if found
        if let Some(ref json_val) = extracted_json {
            println!("\n🔍 Extracted JSON:");
            println!("{}", serde_json::to_string_pretty(json_val).unwrap_or_else(|_| "{}".to_string()));
            
            // Display parsed values
            println!("\n📋 Parsed Values:");
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
                        println!("  - {}: {}", new_prefix, format_value(val));
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
                        println!("  - {}: {}", new_prefix, format_value(val));
                    }
                }
            }
        }
        _ => {
            println!("  - {}: {}", prefix, format_value(value));
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

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Get workflow file from command line args or default to workflow.json
    let workflow_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "workflow.json".to_string());
    
    println!("Loading workflow from: {}", workflow_file);
    
    // Read workflow JSON
    let workflow_data = fs::read_to_string(&workflow_file)
        .map_err(|e| format!("Failed to read workflow file '{}': {}", workflow_file, e))?;
    
    // Parse workflow using dataflow-rs
    let workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;
    
    println!("Loaded workflow: {}", workflow.name);
    println!("Number of tasks: {}", workflow.tasks.len());
    
    // Register custom function handler
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert("cli_command".to_string(), Box::new(CliCommandHandler));
    
    // Create engine with the workflow and custom functions
    let engine = Engine::new(vec![workflow], Some(custom_functions));
    
    // Create initial message with empty context
    let mut message = Message::from_value(&json!({}));
    
    println!("\nExecuting workflow...\n");
    
    // Process the message through the workflow
    match engine.process_message(&mut message).await {
        Ok(_) => {
            println!("\n✅ Workflow execution complete!");
            
            // Display results
            println!("\n📊 Final Context:");
            println!("{}", serde_json::to_string_pretty(&message.context["data"])?);
            
            // Display audit trail
            println!("\n📋 Audit Trail:");
            for (i, audit) in message.audit_trail.iter().enumerate() {
                println!(
                    "{}. Task: {} (Status: {})",
                    i + 1,
                    audit.task_id,
                    audit.status
                );
                println!("   Timestamp: {}", audit.timestamp);
                println!("   Changes: {} field(s) modified", audit.changes.len());
            }
            
            // Check for errors
            if message.has_errors() {
                println!("\n⚠️  Errors encountered:");
                for error in &message.errors {
                    eprintln!(
                        "   - {}: {}",
                        error.task_id.as_ref().unwrap_or(&"unknown".to_string()),
                        error.message
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ Workflow execution failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
