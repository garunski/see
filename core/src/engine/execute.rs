use crate::execution::context::ExecutionContext;
use crate::{AuditEntry, AuditStore, OutputCallback, TaskInfo, WorkflowExecution, WorkflowResult};
use chrono::Utc;
use dataflow_rs::engine::{message::Message, AsyncFunctionHandler};
use dataflow_rs::{Engine, Workflow};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;

use super::handlers::CliCommandHandler;

pub async fn execute_workflow(
    workflow_file: &str,
    output_callback: Option<OutputCallback>,
    store: Option<Arc<dyn AuditStore>>,
) -> Result<WorkflowResult, Box<dyn std::error::Error>> {
    let workflow_data = fs::read_to_string(workflow_file)
        .map_err(|e| format!("Failed to read workflow file '{}': {}", workflow_file, e))?;

    let mut workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;

    for task in &mut workflow.tasks {
        if let dataflow_rs::FunctionConfig::Custom { name, input } = &mut task.function {
            if name == "cli_command" {
                if let Some(input_obj) = input.as_object_mut() {
                    input_obj.insert(
                        "task_id".to_string(),
                        serde_json::Value::String(task.id.clone()),
                    );
                }
            }
        }
    }

    let tasks: Vec<TaskInfo> = workflow
        .tasks
        .iter()
        .map(|task| TaskInfo {
            id: task.id.clone(),
            name: task.name.clone(),
            status: "pending".to_string(),
        })
        .collect();

    let context = ExecutionContext::new(tasks, output_callback);

    context
        .lock()
        .unwrap()
        .log(&format!("Loading workflow from: {}\n", workflow_file));
    context
        .lock()
        .unwrap()
        .log(&format!("Loaded workflow: {}\n", workflow.name));
    context
        .lock()
        .unwrap()
        .log(&format!("Number of tasks: {}\n", workflow.tasks.len()));

    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "cli_command".to_string(),
        Box::new(CliCommandHandler::new(context.clone())),
    );

    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));
    let mut message = Message::from_value(&json!({}));
    let execution_id = Uuid::new_v4().to_string();

    context.lock().unwrap().log("\nExecuting workflow...\n");

    match engine.process_message(&mut message).await {
        Ok(_) => {
            context
                .lock()
                .unwrap()
                .log("\n✅ Workflow execution complete!");

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

            {
                let mut ctx = context.lock().unwrap();
                for audit in &audit_trail {
                    let status = if audit.status == "200" {
                        "complete"
                    } else {
                        "failed"
                    };
                    ctx.update_task_status(&audit.task_id, status);
                }
            }

            context.lock().unwrap().log("\n📋 Audit Trail:");
            for (i, audit) in audit_trail.iter().enumerate() {
                context.lock().unwrap().log(&format!(
                    "{}. Task: {} (Status: {})",
                    i + 1,
                    audit.task_id,
                    audit.status
                ));
                context
                    .lock()
                    .unwrap()
                    .log(&format!("   Timestamp: {}", audit.timestamp));
                context.lock().unwrap().log(&format!(
                    "   Changes: {} field(s) modified",
                    audit.changes_count
                ));
            }

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

                context.lock().unwrap().log("\n⚠️  Errors encountered:");
                for error in &error_list {
                    context.lock().unwrap().log(&format!("   - {}", error));
                }
                error_list
            } else {
                Vec::new()
            };

            let (output_logs, per_task_logs, tasks) = match Arc::try_unwrap(context) {
                Ok(context) => context
                    .into_inner()
                    .map_err(|e| format!("Failed to lock context: {:?}", e))?
                    .extract_data(),
                Err(context) => {
                    let ctx = context.lock().unwrap();
                    (
                        ctx.get_output_logs(),
                        ctx.get_per_task_logs(),
                        ctx.get_tasks(),
                    )
                }
            };

            let result = WorkflowResult {
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
            };

            // Save to database if store is provided
            if let Some(store) = store {
                let execution = WorkflowExecution {
                    id: result.execution_id.clone(),
                    workflow_name: result.workflow_name.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    success: result.success,
                    tasks: result.tasks.clone(),
                    audit_trail: result.audit_trail.clone(),
                    per_task_logs: result.per_task_logs.clone(),
                    errors: result.errors.clone(),
                };

                if let Err(e) = store.save_workflow_execution(&execution).await {
                    // Log warning if we can't save to database
                    println!(
                        "⚠️  Warning: Failed to save workflow execution to database: {}",
                        e
                    );
                }
            }

            Ok(result)
        }
        Err(e) => {
            context
                .lock()
                .unwrap()
                .log(&format!("\n❌ Workflow execution failed: {}", e));
            Err(e.into())
        }
    }
}
