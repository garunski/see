use crate::execution::context::{ExecutionContext, ExecutionContextSafe};
use crate::{
    errors::CoreError, AuditEntry, AuditStatus, AuditStore, OutputCallback, TaskInfo, TaskStatus,
    WorkflowExecution, WorkflowResult,
};
use chrono::Utc;
use dataflow_rs::engine::{message::Message, AsyncFunctionHandler};
use dataflow_rs::{Engine, Workflow};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

use super::handlers::CliCommandHandler;

pub async fn execute_workflow_from_content(
    workflow_data: &str,
    output_callback: Option<OutputCallback>,
    store: Option<Arc<dyn AuditStore>>,
) -> Result<WorkflowResult, CoreError> {
    let mut workflow = Workflow::from_json(workflow_data)
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
            status: TaskStatus::Pending,
        })
        .collect();

    let execution_id = Uuid::new_v4().to_string();
    let workflow_start_time = chrono::Utc::now().to_rfc3339();

    let context = ExecutionContext::new(
        tasks.clone(),
        output_callback,
        store.clone(),
        execution_id.clone(),
        workflow.name.clone(),
    );

    if let Err(e) = context.safe_log("Loading workflow from content\n") {
        eprintln!("Failed to log workflow loading: {}", e);
    }
    if let Err(e) = context.safe_log(&format!("Loaded workflow: {}\n", workflow.name)) {
        eprintln!("Failed to log workflow name: {}", e);
    }
    if let Err(e) = context.safe_log(&format!("Number of tasks: {}\n", workflow.tasks.len())) {
        eprintln!("Failed to log task count: {}", e);
    }

    // Save workflow metadata at start
    if let Some(ref store) = store {
        let metadata = crate::persistence::models::WorkflowMetadata {
            id: execution_id.clone(),
            workflow_name: workflow.name.clone(),
            start_timestamp: workflow_start_time.clone(),
            end_timestamp: None,
            status: crate::persistence::models::WorkflowStatus::Running,
            task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
        };
        if let Err(e) = store.save_workflow_metadata(&metadata).await {
            eprintln!("Failed to save workflow metadata: {}", e);
        }
    }

    // Emit execution_id to GUI for polling
    if let Err(e) = context.safe_log(&format!("EXECUTION_ID:{}\n", execution_id)) {
        eprintln!("Failed to emit execution_id: {}", e);
    }

    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "cli_command".to_string(),
        Box::new(CliCommandHandler::new(context.clone())),
    );

    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));
    let mut message = Message::from_value(&json!({}));

    if let Err(e) = context.safe_log("\nExecuting workflow...\n") {
        eprintln!("Failed to log execution start: {}", e);
    }

    match engine.process_message(&mut message).await {
        Ok(_) => {
            if let Err(e) = context.safe_log("\n✅ Workflow execution complete!") {
                eprintln!("Failed to log execution completion: {}", e);
            }

            let audit_trail: Vec<AuditEntry> = message
                .audit_trail
                .iter()
                .map(|audit| AuditEntry {
                    task_id: audit.task_id.to_string(),
                    status: AuditStatus::from_http_code(&audit.status.to_string()),
                    timestamp: audit.timestamp.to_string(),
                    changes_count: audit.changes.len(),
                })
                .collect();

            for audit in &audit_trail {
                let status = match audit.status {
                    AuditStatus::Success => TaskStatus::Complete,
                    AuditStatus::Failure => TaskStatus::Failed,
                };
                if let Err(e) = context.safe_update_task_status(&audit.task_id, status) {
                    eprintln!("Failed to update task status: {}", e);
                }
            }

            if let Err(e) = context.safe_log("\n📋 Audit Trail:") {
                eprintln!("Failed to log audit trail header: {}", e);
            }
            for (i, audit) in audit_trail.iter().enumerate() {
                if let Err(e) = context.safe_log(&format!(
                    "{}. Task: {} (Status: {})",
                    i + 1,
                    audit.task_id,
                    audit.status
                )) {
                    eprintln!("Failed to log audit entry: {}", e);
                }
                if let Err(e) = context.safe_log(&format!("   Timestamp: {}", audit.timestamp)) {
                    eprintln!("Failed to log audit timestamp: {}", e);
                }
                if let Err(e) = context.safe_log(&format!(
                    "   Changes: {} field(s) modified",
                    audit.changes_count
                )) {
                    eprintln!("Failed to log audit changes: {}", e);
                }
            }

            let errors: Vec<String> = if message.has_errors() {
                let error_list: Vec<String> = message
                    .errors
                    .iter()
                    .map(|error| {
                        format!(
                            "Task {}: {}",
                            error.task_id.as_ref().unwrap_or(&"unknown".to_string()),
                            error.message
                        )
                    })
                    .collect();

                if let Err(e) = context.safe_log("\n⚠️  Errors encountered:") {
                    eprintln!("Failed to log errors header: {}", e);
                }
                for error in &error_list {
                    if let Err(e) = context.safe_log(&format!("   - {}", error)) {
                        eprintln!("Failed to log error: {}", e);
                    }
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
                workflow_name: workflow.name.clone(),
                task_count: workflow.tasks.len(),
                execution_id: execution_id.clone(),
                tasks: tasks.clone(),
                final_context: message.context["data"].clone(),
                audit_trail: audit_trail.clone(),
                per_task_logs: per_task_logs.clone(),
                errors: errors.clone(),
                output_logs: output_logs.clone(),
            };

            // Save workflow completion metadata
            if let Some(ref store) = store {
                let metadata = crate::persistence::models::WorkflowMetadata {
                    id: execution_id.clone(),
                    workflow_name: workflow.name.clone(),
                    start_timestamp: workflow_start_time.clone(),
                    end_timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    status: crate::persistence::models::WorkflowStatus::Complete,
                    task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
                };
                if let Err(e) = store.save_workflow_metadata(&metadata).await {
                    eprintln!("Failed to update workflow completion: {}", e);
                }
            }

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
            if let Err(log_err) =
                context.safe_log(&format!("\n❌ Workflow execution failed: {}", e))
            {
                eprintln!("Failed to log execution failure: {}", log_err);
            }

            // Save workflow failure metadata
            if let Some(ref store) = store {
                let metadata = crate::persistence::models::WorkflowMetadata {
                    id: execution_id.clone(),
                    workflow_name: workflow.name.clone(),
                    start_timestamp: workflow_start_time.clone(),
                    end_timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    status: crate::persistence::models::WorkflowStatus::Failed,
                    task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
                };
                if let Err(e) = store.save_workflow_metadata(&metadata).await {
                    eprintln!("Failed to update workflow failure: {}", e);
                }
            }

            Err(CoreError::WorkflowExecution(format!(
                "Workflow execution failed: {}",
                e
            )))
        }
    }
}

pub async fn execute_workflow(
    workflow_file: &str,
    output_callback: Option<OutputCallback>,
    store: Option<Arc<dyn AuditStore>>,
) -> Result<WorkflowResult, CoreError> {
    let workflow_data = fs::read_to_string(workflow_file).await.map_err(|e| {
        CoreError::WorkflowExecution(format!(
            "Failed to read workflow file '{}': {}",
            workflow_file, e
        ))
    })?;

    execute_workflow_from_content(&workflow_data, output_callback, store).await
}
