pub mod json_parser;
pub mod cli_handler;

use dataflow_rs::{Engine, Workflow};
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    message::Message,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

/// Output callback type for real-time output during workflow execution
pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Workflow execution result
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub final_context: serde_json::Value,
    pub audit_trail: Vec<AuditEntry>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
}

/// Audit trail entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub task_id: String,
    pub status: String,
    pub timestamp: String,
    pub changes_count: usize,
}

/// Execute a workflow from a JSON file
pub async fn execute_workflow(
    workflow_file: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, Box<dyn std::error::Error>> {
    let mut output_logs = Vec::new();
    
    // Clone the callback for the CLI handler
    let cli_callback = output_callback.clone();
    
    // Helper to log output
    let log_output = |msg: &str, output_logs: &mut Vec<String>| {
        if let Some(ref callback) = output_callback {
            callback(msg.to_string());
        }
        output_logs.push(msg.to_string());
    };
    
    // Set up CLI handler callback if provided
    if let Some(callback) = cli_callback {
        cli_handler::set_output_callback(callback);
    }

    log_output(&format!("Loading workflow from: {}\n", workflow_file), &mut output_logs);
    
    // Read workflow JSON
    let workflow_data = fs::read_to_string(workflow_file)
        .map_err(|e| format!("Failed to read workflow file '{}': {}", workflow_file, e))?;
    
    // Parse workflow using dataflow-rs
    let workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;
    
    log_output(&format!("Loaded workflow: {}\n", workflow.name), &mut output_logs);
    log_output(&format!("Number of tasks: {}\n", workflow.tasks.len()), &mut output_logs);
    
    // Register custom function handler
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert("cli_command".to_string(), Box::new(cli_handler::CliCommandHandler));
    
    // Create engine with the workflow and custom functions
    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));
    
    // Create initial message with empty context
    let mut message = Message::from_value(&json!({}));
    
    log_output("\nExecuting workflow...\n", &mut output_logs);
    
    // Process the message through the workflow
    match engine.process_message(&mut message).await {
        Ok(_) => {
            log_output("\n✅ Workflow execution complete!", &mut output_logs);
            
            // Build audit trail
            let audit_trail: Vec<AuditEntry> = message.audit_trail.iter().map(|audit| {
                AuditEntry {
                    task_id: audit.task_id.to_string(),
                    status: audit.status.to_string(),
                    timestamp: audit.timestamp.to_string(),
                    changes_count: audit.changes.len(),
                }
            }).collect();
            
            // Log audit trail
            log_output("\n📋 Audit Trail:", &mut output_logs);
            for (i, audit) in audit_trail.iter().enumerate() {
                log_output(&format!(
                    "{}. Task: {} (Status: {})",
                    i + 1,
                    audit.task_id,
                    audit.status
                ), &mut output_logs);
                log_output(&format!("   Timestamp: {}", audit.timestamp), &mut output_logs);
                log_output(&format!("   Changes: {} field(s) modified", audit.changes_count), &mut output_logs);
            }
            
            // Check for errors
            let errors: Vec<String> = if message.has_errors() {
                let error_list: Vec<String> = message.errors.iter().map(|error| {
                    format!(
                        "{}: {}",
                        error.task_id.as_ref().unwrap_or(&"unknown".to_string()),
                        error.message
                    )
                }).collect();
                
                log_output("\n⚠️  Errors encountered:", &mut output_logs);
                for error in &error_list {
                    log_output(&format!("   - {}", error), &mut output_logs);
                }
                
                error_list
            } else {
                Vec::new()
            };
            
            // Clear the callback
            cli_handler::clear_output_callback();
            
            Ok(WorkflowResult {
                success: errors.is_empty(),
                workflow_name: workflow.name,
                task_count: workflow.tasks.len(),
                final_context: message.context["data"].clone(),
                audit_trail,
                errors,
                output_logs,
            })
        }
        Err(e) => {
            let error_msg = format!("\n❌ Workflow execution failed: {}", e);
            log_output(&error_msg, &mut output_logs);
            cli_handler::clear_output_callback();
            Err(e.into())
        }
    }
}
