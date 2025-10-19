pub mod cli_handler;
pub mod components;
pub mod db;
pub mod json_parser;

use dataflow_rs::engine::{message::Message, AsyncFunctionHandler};
use dataflow_rs::{Engine, Workflow};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

/// Output callback type for real-time output during workflow execution
pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Information about a single task in a workflow
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: String, // pending, in-progress, complete, failed
}

/// Workflow execution result
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub execution_id: String,
    pub tasks: Vec<TaskInfo>,
    pub final_context: serde_json::Value,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
}

/// Audit trail entry
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    let mut per_task_logs: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_task_id: Option<String> = None;

    // Clone the callback for the CLI handler
    let cli_callback = output_callback.clone();

    // Helper to log output with per-task tracking
    let mut log_output = |msg: &str, output_logs: &mut Vec<String>, per_task_logs: &mut HashMap<String, Vec<String>>, current_task_id: &mut Option<String>, tasks: &mut Vec<TaskInfo>| {
        // Parse task boundary markers
        if msg.starts_with("[TASK_START:") && msg.ends_with("]") {
            let task_id = msg.strip_prefix("[TASK_START:").unwrap().strip_suffix("]").unwrap().to_string();
            *current_task_id = Some(task_id.clone());
            
            // Update task status to in-progress
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = "in-progress".to_string();
            }
            return;
        }
        
        if msg.starts_with("[TASK_END:") && msg.ends_with("]") {
            let task_id = msg.strip_prefix("[TASK_END:").unwrap().strip_suffix("]").unwrap().to_string();
            *current_task_id = None;
            return;
        }
        
        if let Some(ref callback) = output_callback {
            callback(msg.to_string());
        }
        output_logs.push(msg.to_string());
        
        // Track per-task logs
        if let Some(ref task_id) = current_task_id {
            per_task_logs.entry(task_id.clone()).or_insert_with(Vec::new).push(msg.to_string());
        }
    };

    // Set up CLI handler callback if provided
    if let Some(callback) = cli_callback {
        cli_handler::set_output_callback(callback);
    }

    // Read workflow JSON
    let workflow_data = fs::read_to_string(workflow_file)
        .map_err(|e| format!("Failed to read workflow file '{}': {}", workflow_file, e))?;

    // Parse workflow using dataflow-rs
    let workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;

    // Extract task information from workflow
    let mut tasks: Vec<TaskInfo> = workflow.tasks.iter().map(|task| {
        TaskInfo {
            id: task.id.clone(),
            name: task.name.clone(),
            status: "pending".to_string(),
        }
    }).collect();

    log_output(
        &format!("Loading workflow from: {}\n", workflow_file),
        &mut output_logs,
        &mut per_task_logs,
        &mut current_task_id,
        &mut tasks,
    );

    log_output(
        &format!("Loaded workflow: {}\n", workflow.name),
        &mut output_logs,
        &mut per_task_logs,
        &mut current_task_id,
        &mut tasks,
    );
    log_output(
        &format!("Number of tasks: {}\n", workflow.tasks.len()),
        &mut output_logs,
        &mut per_task_logs,
        &mut current_task_id,
        &mut tasks,
    );

    // Register custom function handler
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "cli_command".to_string(),
        Box::new(cli_handler::CliCommandHandler),
    );

    // Create engine with the workflow and custom functions
    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));

    // Create initial message with empty context
    let mut message = Message::from_value(&json!({}));

    // Generate execution ID
    let execution_id = Uuid::new_v4().to_string();

    log_output("\nExecuting workflow...\n", &mut output_logs, &mut per_task_logs, &mut current_task_id, &mut tasks);

    // Process the message through the workflow
    match engine.process_message(&mut message).await {
        Ok(_) => {
            log_output("\n✅ Workflow execution complete!", &mut output_logs, &mut per_task_logs, &mut current_task_id, &mut tasks);

            // Build audit trail
            let audit_trail: Vec<AuditEntry> = message
                .audit_trail
                .iter()
                .map(|audit| AuditEntry {
                    task_id: audit.task_id.to_string(),
                    status: audit.status.to_string(),
                    timestamp: audit.timestamp.to_string(),
                    changes_count: audit.changes.len(),
                })
                .collect();

            // Update task statuses based on audit trail
            for audit in &audit_trail {
                if let Some(task) = tasks.iter_mut().find(|t| t.id == audit.task_id) {
                    task.status = if audit.status == "200" { "complete".to_string() } else { "failed".to_string() };
                }
            }

            // Log audit trail
            log_output("\n📋 Audit Trail:", &mut output_logs, &mut per_task_logs, &mut current_task_id, &mut tasks);
            for (i, audit) in audit_trail.iter().enumerate() {
                log_output(
                    &format!(
                        "{}. Task: {} (Status: {})",
                        i + 1,
                        audit.task_id,
                        audit.status
                    ),
                    &mut output_logs,
                    &mut per_task_logs,
                    &mut current_task_id,
                    &mut tasks,
                );
                log_output(
                    &format!("   Timestamp: {}", audit.timestamp),
                    &mut output_logs,
                    &mut per_task_logs,
                    &mut current_task_id,
                    &mut tasks,
                );
                log_output(
                    &format!("   Changes: {} field(s) modified", audit.changes_count),
                    &mut output_logs,
                    &mut per_task_logs,
                    &mut current_task_id,
                    &mut tasks,
                );
            }

            // Check for errors
            let errors: Vec<String> = if message.has_errors() {
                let error_list: Vec<String> = message
                    .errors
                    .iter()
                    .map(|error| {
                        format!(
                            "{}: {}",
                            error.task_id.as_ref().unwrap_or(&"unknown".to_string()),
                            error.message
                        )
                    })
                    .collect();

                log_output("\n⚠️  Errors encountered:", &mut output_logs, &mut per_task_logs, &mut current_task_id, &mut tasks);
                for error in &error_list {
                    log_output(&format!("   - {}", error), &mut output_logs, &mut per_task_logs, &mut current_task_id, &mut tasks);
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
                execution_id,
                tasks,
                final_context: message.context["data"].clone(),
                audit_trail,
                per_task_logs,
                errors,
                output_logs,
            })
        }
        Err(e) => {
            let error_msg = format!("\n❌ Workflow execution failed: {}", e);
            log_output(&error_msg, &mut output_logs, &mut per_task_logs, &mut current_task_id, &mut tasks);
            cli_handler::clear_output_callback();
            Err(e.into())
        }
    }
}
