pub mod cli_handler;
pub mod components;
pub mod db;
pub mod execution_context;
pub mod json_parser;

use dataflow_rs::engine::{message::Message, AsyncFunctionHandler};
use dataflow_rs::{Engine, Workflow};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;

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
    // Read and parse workflow
    let workflow_data = fs::read_to_string(workflow_file)
        .map_err(|e| format!("Failed to read workflow file '{}': {}", workflow_file, e))?;
    
    let mut workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;
    
    // Inject task IDs into CLI commands (necessary for task tracking)
    for task in &mut workflow.tasks {
        if let dataflow_rs::FunctionConfig::Custom { name, input } = &mut task.function {
            if name == "cli_command" {
                if let Some(input_obj) = input.as_object_mut() {
                    input_obj.insert("task_id".to_string(), serde_json::Value::String(task.id.clone()));
                }
            }
        }
    }
    
    // Extract task info
    let tasks: Vec<TaskInfo> = workflow.tasks.iter()
        .map(|task| TaskInfo {
            id: task.id.clone(),
            name: task.name.clone(),
            status: "pending".to_string(),
        })
        .collect();
    
    // Create execution context
    let context = execution_context::ExecutionContext::new(tasks, output_callback);
    
    // Log initial messages
    context.lock().unwrap().log(&format!("Loading workflow from: {}\n", workflow_file));
    context.lock().unwrap().log(&format!("Loaded workflow: {}\n", workflow.name));
    context.lock().unwrap().log(&format!("Number of tasks: {}\n", workflow.tasks.len()));
    
    // Register custom function handler with context
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> = HashMap::new();
    custom_functions.insert(
        "cli_command".to_string(),
        Box::new(cli_handler::CliCommandHandler::new(context.clone())),
    );
    
    // Create engine
    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));
    let mut message = Message::from_value(&json!({}));
    let execution_id = Uuid::new_v4().to_string();
    
    context.lock().unwrap().log("\nExecuting workflow...\n");
    
    // Execute workflow
    match engine.process_message(&mut message).await {
        Ok(_) => {
            context.lock().unwrap().log("\n✅ Workflow execution complete!");
            
            // Build audit trail
            let audit_trail: Vec<AuditEntry> = message.audit_trail.iter()
                .map(|audit| AuditEntry {
                    task_id: audit.task_id.to_string(),
                    status: audit.status.to_string(),
                    timestamp: audit.timestamp.to_string(),
                    changes_count: audit.changes.len(),
                })
                .collect();
            
            // Update task statuses from audit trail
            {
                let mut ctx = context.lock().unwrap();
                for audit in &audit_trail {
                    let status = if audit.status == "200" { "complete" } else { "failed" };
                    ctx.update_task_status(&audit.task_id, status);
                }
            }
            
            // Log audit trail
            context.lock().unwrap().log("\n📋 Audit Trail:");
            for (i, audit) in audit_trail.iter().enumerate() {
                context.lock().unwrap().log(&format!("{}. Task: {} (Status: {})", i + 1, audit.task_id, audit.status));
                context.lock().unwrap().log(&format!("   Timestamp: {}", audit.timestamp));
                context.lock().unwrap().log(&format!("   Changes: {} field(s) modified", audit.changes_count));
            }
            
            // Collect errors
            let errors: Vec<String> = if message.has_errors() {
                let error_list: Vec<String> = message.errors.iter()
                    .map(|error| format!("{}: {}", error.task_id.as_ref().unwrap_or(&"unknown".to_string()), error.message))
                    .collect();
                
                context.lock().unwrap().log("\n⚠️  Errors encountered:");
                for error in &error_list {
                    context.lock().unwrap().log(&format!("   - {}", error));
                }
                error_list
            } else {
                Vec::new()
            };
            
            // Extract data from context
            let (output_logs, per_task_logs, tasks) = match Arc::try_unwrap(context) {
                Ok(context) => {
                    // Successfully unwrapped, extract data
                    context.into_inner()
                        .map_err(|e| format!("Failed to lock context: {:?}", e))?
                        .extract_data()
                }
                Err(context) => {
                    // Context still in use, clone the data instead
                    let ctx = context.lock().unwrap();
                    (ctx.get_output_logs(), ctx.get_per_task_logs(), ctx.get_tasks())
                }
            };
            
            // Debug output
            println!("DEBUG: per_task_logs keys: {:?}", per_task_logs.keys().collect::<Vec<_>>());
            for (task_id, logs) in &per_task_logs {
                println!("DEBUG: Task {} has {} logs", task_id, logs.len());
            }
            
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
            context.lock().unwrap().log(&format!("\n❌ Workflow execution failed: {}", e));
            Err(e.into())
        }
    }
}
