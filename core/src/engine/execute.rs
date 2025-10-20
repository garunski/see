use crate::execution::context::{ExecutionContext, ExecutionContextSafe};
use crate::{
    errors::CoreError, AuditEntry, AuditStore, OutputCallback, TaskInfo, WorkflowExecution,
    WorkflowResult,
};
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
) -> Result<WorkflowResult, CoreError> {
    let workflow_data = fs::read_to_string(workflow_file).map_err(|e| {
        CoreError::WorkflowExecution(format!(
            "Failed to read workflow file '{}': {}",
            workflow_file, e
        ))
    })?;

    let mut workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| CoreError::WorkflowExecution(format!("Failed to parse workflow: {}", e)))?;

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
        .safe_log(&format!("Loading workflow from: {}\n", workflow_file))
        .map_err(|e| {
            CoreError::WorkflowExecution(format!("Failed to log workflow loading: {}", e))
        })?;
    context
        .safe_log(&format!("Loaded workflow: {}\n", workflow.name))
        .map_err(|e| CoreError::WorkflowExecution(format!("Failed to log workflow name: {}", e)))?;
    context
        .safe_log(&format!("Number of tasks: {}\n", workflow.tasks.len()))
        .map_err(|e| CoreError::WorkflowExecution(format!("Failed to log task count: {}", e)))?;

    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "cli_command".to_string(),
        Box::new(CliCommandHandler::new(context.clone())),
    );

    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));
    let mut message = Message::from_value(&json!({}));
    let execution_id = Uuid::new_v4().to_string();

    context.safe_log("\nExecuting workflow...\n").map_err(|e| {
        CoreError::WorkflowExecution(format!("Failed to log execution start: {}", e))
    })?;

    match engine.process_message(&mut message).await {
        Ok(_) => {
            context
                .safe_log("\n✅ Workflow execution complete!")
                .map_err(|e| {
                    CoreError::WorkflowExecution(format!(
                        "Failed to log execution completion: {}",
                        e
                    ))
                })?;

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

            for audit in &audit_trail {
                let status = if audit.status == "200" {
                    "complete"
                } else {
                    "failed"
                };
                context
                    .safe_update_task_status(&audit.task_id, status)
                    .map_err(|e| {
                        CoreError::WorkflowExecution(format!("Failed to update task status: {}", e))
                    })?;
            }

            context.safe_log("\n📋 Audit Trail:").map_err(|e| {
                CoreError::WorkflowExecution(format!("Failed to log audit trail header: {}", e))
            })?;
            for (i, audit) in audit_trail.iter().enumerate() {
                context
                    .safe_log(&format!(
                        "{}. Task: {} (Status: {})",
                        i + 1,
                        audit.task_id,
                        audit.status
                    ))
                    .map_err(|e| {
                        CoreError::WorkflowExecution(format!("Failed to log audit entry: {}", e))
                    })?;
                context
                    .safe_log(&format!("   Timestamp: {}", audit.timestamp))
                    .map_err(|e| {
                        CoreError::WorkflowExecution(format!(
                            "Failed to log audit timestamp: {}",
                            e
                        ))
                    })?;
                context
                    .safe_log(&format!(
                        "   Changes: {} field(s) modified",
                        audit.changes_count
                    ))
                    .map_err(|e| {
                        CoreError::WorkflowExecution(format!("Failed to log audit changes: {}", e))
                    })?;
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

                context.safe_log("\n⚠️  Errors encountered:").map_err(|e| {
                    CoreError::WorkflowExecution(format!("Failed to log errors header: {}", e))
                })?;
                for error in &error_list {
                    context.safe_log(&format!("   - {}", error)).map_err(|e| {
                        CoreError::WorkflowExecution(format!("Failed to log error: {}", e))
                    })?;
                }
                error_list
            } else {
                Vec::new()
            };

            let (output_logs, per_task_logs, tasks) = match Arc::try_unwrap(context.clone()) {
                Ok(context) => context
                    .into_inner()
                    .map_err(|e| {
                        CoreError::ExecutionContext(format!("Failed to unwrap context: {:?}", e))
                    })?
                    .extract_data(),
                Err(context) => {
                    let ctx = context.lock().map_err(|e| {
                        CoreError::MutexLock(format!(
                            "Failed to lock context for data extraction: {}",
                            e
                        ))
                    })?;
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
                    // Log error but don't fail the workflow
                    context
                        .safe_log(&format!(
                            "⚠️  Warning: Failed to save workflow execution to database: {}",
                            e
                        ))
                        .unwrap_or_else(|log_err| {
                            eprintln!("Failed to log database error: {}", log_err)
                        });
                }
            }

            Ok(result)
        }
        Err(e) => {
            context
                .safe_log(&format!("\n❌ Workflow execution failed: {}", e))
                .unwrap_or_else(|log_err| {
                    eprintln!("Failed to log execution failure: {}", log_err)
                });
            Err(CoreError::WorkflowExecution(format!(
                "Workflow execution failed: {}",
                e
            )))
        }
    }
}
