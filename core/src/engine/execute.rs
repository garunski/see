use crate::execution::context::{ExecutionContext, ExecutionContextSafe};
use crate::{
    errors::CoreError,
    types::{AuditEntry, AuditStatus, OutputCallback, TaskInfo, TaskStatus, WorkflowResult},
    WorkflowExecution,
};
use chrono::Utc;
use dataflow_rs::engine::{message::Message, AsyncFunctionHandler};
use dataflow_rs::{Engine, Workflow};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use super::handlers::{CliCommandHandler, CursorAgentHandler};

#[instrument(skip(workflow_data, output_callback), fields(execution_id))]
pub async fn execute_workflow_from_content(
    workflow_data: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    let store = crate::get_global_store()?;
    let mut workflow = Workflow::from_json(workflow_data)
        .map_err(|e| CoreError::WorkflowExecution(format!("Failed to parse workflow: {}", e)))?;

    for task in &mut workflow.tasks {
        if let dataflow_rs::FunctionConfig::Custom { name, input } = &mut task.function {
            if name == "cli_command" || name == "cursor_agent" {
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
    tracing::Span::current().record("execution_id", &execution_id);
    info!(
        execution_id = %execution_id,
        workflow_name = %workflow.name,
        task_count = workflow.tasks.len(),
        "Starting workflow execution - this execution will continue even if user navigates away from home page"
    );
    let workflow_start_time = chrono::Utc::now().to_rfc3339();

    let context = ExecutionContext::new(
        tasks.clone(),
        output_callback,
        Some(store.clone()),
        execution_id.clone(),
        workflow.name.clone(),
    );

    if let Err(e) = context.safe_log("Loading workflow from content\n") {
        error!(error = %e, "Failed to log workflow loading");
    }
    if let Err(e) = context.safe_log(&format!("Loaded workflow: {}\n", workflow.name)) {
        error!(error = %e, "Failed to log workflow name");
    }
    if let Err(e) = context.safe_log(&format!("Number of tasks: {}\n", workflow.tasks.len())) {
        error!(error = %e, "Failed to log task count");
    }

    {
        let metadata = crate::persistence::models::WorkflowMetadata {
            id: execution_id.clone(),
            workflow_name: workflow.name.clone(),
            start_timestamp: workflow_start_time.clone(),
            end_timestamp: None,
            status: crate::persistence::models::WorkflowStatus::Running,
            task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
            is_paused: false,
            paused_task_id: None,
        };
        if let Err(e) = store.save_workflow_metadata(&metadata).await {
            error!(error = %e, "Failed to save workflow metadata");
        }

        for task in &tasks {
            let task_exec = crate::persistence::models::TaskExecution {
                execution_id: execution_id.clone(),
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                status: TaskStatus::Pending,
                logs: Vec::new(),
                start_timestamp: String::new(),
                end_timestamp: String::new(),
            };
            if let Err(e) = store.save_task_execution(&task_exec).await {
                eprintln!(
                    "Failed to save initial task execution for task {}: {}",
                    task.id, e
                );
            }
        }
    }

    if let Err(e) = context.safe_log(&format!("EXECUTION_ID:{}\n", execution_id)) {
        eprintln!("Failed to emit execution_id: {}", e);
    }

    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "cli_command".to_string(),
        Box::new(CliCommandHandler::new(context.clone())),
    );
    custom_functions.insert(
        "cursor_agent".to_string(),
        Box::new(CursorAgentHandler::new(context.clone())),
    );

    let engine = Engine::new(vec![workflow.clone()], Some(custom_functions));
    let mut message = Message::from_value(&json!({}));

    debug!(
        execution_id = %execution_id,
        workflow_name = %workflow.name,
        task_count = workflow.tasks.len(),
        "Executing workflow via dataflow engine"
    );
    if let Err(e) = context.safe_log("\nExecuting workflow...\n") {
        error!(
            error = %e,
            execution_id = %execution_id,
            "Failed to log execution start - context may be lost"
        );
    }

    match engine.process_message(&mut message).await {
        Ok(_) => {
            info!(
                execution_id = %execution_id,
                workflow_name = %workflow.name,
                "Workflow execution completed successfully"
            );
            if let Err(e) = context.safe_log("\n✅ Workflow execution complete!") {
                error!(
                    error = %e,
                    execution_id = %execution_id,
                    "Failed to log execution completion - context may be lost"
                );
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
                Ok(context) => {
                    debug!(
                        execution_id = %execution_id,
                        "Successfully unwrapped execution context - single reference"
                    );
                    context
                        .into_inner()
                        .map_err(|e| {
                            error!(
                                error = %e,
                                execution_id = %execution_id,
                                "Failed to unwrap context - this may indicate workflow interruption"
                            );
                            CoreError::ExecutionContext(format!(
                                "Failed to unwrap context: {:?}",
                                e
                            ))
                        })?
                        .extract_data()
                }
                Err(context) => {
                    debug!(
                        execution_id = %execution_id,
                        "Multiple references to execution context - using lock"
                    );
                    let ctx = context.lock().map_err(|e| {
                        error!(
                            error = %e,
                            execution_id = %execution_id,
                            "Failed to lock context for data extraction - this may indicate workflow interruption"
                        );
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

            {
                info!(
                    execution_id = %execution_id,
                    workflow_name = %workflow.name,
                    "Saving successful workflow execution to database"
                );
                let metadata = crate::persistence::models::WorkflowMetadata {
                    id: execution_id.clone(),
                    workflow_name: workflow.name.clone(),
                    start_timestamp: workflow_start_time.clone(),
                    end_timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    status: crate::persistence::models::WorkflowStatus::Complete,
                    task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
                    is_paused: false,
                    paused_task_id: None,
                };
                if let Err(e) = store.save_workflow_metadata(&metadata).await {
                    error!(error = %e, "Failed to save workflow completion metadata");
                }
            }

            {
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

            info!(
                execution_id = %execution_id,
                workflow_name = %workflow.name,
                success = result.success,
                "Workflow execution completed successfully - returning result"
            );
            Ok(result)
        }
        Err(e) => {
            error!(
                error = %e,
                execution_id = %execution_id,
                workflow_name = %workflow.name,
                "Workflow execution failed - this may be due to navigation away from home page or other interruption"
            );
            if let Err(log_err) =
                context.safe_log(&format!("\n❌ Workflow execution failed: {}", e))
            {
                error!(
                    error = %log_err,
                    execution_id = %execution_id,
                    "Failed to log execution failure - context may be lost"
                );
            }

            {
                info!(
                    execution_id = %execution_id,
                    workflow_name = %workflow.name,
                    "Saving failed workflow execution to database"
                );
                let metadata = crate::persistence::models::WorkflowMetadata {
                    id: execution_id.clone(),
                    workflow_name: workflow.name.clone(),
                    start_timestamp: workflow_start_time.clone(),
                    end_timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    status: crate::persistence::models::WorkflowStatus::Failed,
                    task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
                    is_paused: false,
                    paused_task_id: None,
                };
                if let Err(e) = store.save_workflow_metadata(&metadata).await {
                    eprintln!("Failed to update workflow failure: {}", e);
                }
            }

            error!(
                execution_id = %execution_id,
                workflow_name = %workflow.name,
                "Workflow execution failed - returning error result"
            );
            Err(CoreError::WorkflowExecution(format!(
                "Workflow execution failed: {}",
                e
            )))
        }
    }
}

#[instrument(skip(output_callback), fields(workflow_file = %workflow_file))]
pub async fn execute_workflow(
    workflow_file: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    debug!("Reading workflow file");
    let workflow_data = fs::read_to_string(workflow_file).await.map_err(|e| {
        CoreError::WorkflowExecution(format!(
            "Failed to read workflow file '{}': {}",
            workflow_file, e
        ))
    })?;

    execute_workflow_from_content(&workflow_data, output_callback).await
}

#[instrument(skip(output_callback), fields(workflow_id = %workflow_id))]
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    debug!("Fetching workflow definition by ID");
    let store = crate::get_global_store()?;
    let workflow_definition = store.get_workflow_definition(workflow_id).await?;

    debug!(
        workflow_id = %workflow_id,
        workflow_name = %workflow_definition.get_name(),
        "Found workflow definition, executing"
    );

    execute_workflow_from_content(&workflow_definition.content, output_callback).await
}

/// Resume a workflow that is waiting for user input
#[instrument(skip(execution_id), fields(execution_id))]
pub async fn resume_workflow(execution_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;

    info!(
        execution_id = %execution_id,
        "Resuming workflow execution"
    );

    // Load workflow metadata
    let metadata = store.get_workflow_metadata(execution_id).await?;

    // Check if workflow is actually waiting for input
    if metadata.status != crate::persistence::models::WorkflowStatus::WaitingForInput {
        return Err(CoreError::Validation(format!(
            "Workflow {} is not waiting for input (status: {:?})",
            execution_id, metadata.status
        )));
    }

    // Load task executions
    let task_executions = store.get_task_executions(execution_id).await?;

    // Find tasks that are waiting for input
    let waiting_tasks: Vec<_> = task_executions
        .iter()
        .filter(|task| task.status == crate::types::TaskStatus::WaitingForInput)
        .collect();

    if waiting_tasks.is_empty() {
        return Err(CoreError::Validation(format!(
            "No tasks waiting for input in workflow {}",
            execution_id
        )));
    }

    // Resume each waiting task
    for task in waiting_tasks {
        let mut updated_task = task.clone();
        updated_task.status = crate::types::TaskStatus::InProgress;
        updated_task.end_timestamp = String::new(); // Clear end timestamp

        store.save_task_execution(&updated_task).await?;

        info!(
            execution_id = %execution_id,
            task_id = %task.task_id,
            "Resumed task from waiting state"
        );
    }

    // Mark workflow as resumed using new persistence method
    store.mark_workflow_resumed(execution_id).await?;

    info!(
        execution_id = %execution_id,
        "Workflow resumed successfully"
    );

    Ok(())
}

/// Resume a specific task that is waiting for user input
#[instrument(skip(execution_id, task_id), fields(execution_id, task_id))]
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Resuming specific task"
    );

    // Load task execution
    let task_executions = store.get_task_executions(execution_id).await?;
    let task = task_executions
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| {
            CoreError::Validation(format!(
                "Task {} not found in execution {}",
                task_id, execution_id
            ))
        })?;

    // Check if task is waiting for input
    if task.status != crate::types::TaskStatus::WaitingForInput {
        return Err(CoreError::Validation(format!(
            "Task {} is not waiting for input (status: {})",
            task_id, task.status
        )));
    }

    // Update task status
    let mut updated_task = task.clone();
    updated_task.status = crate::types::TaskStatus::InProgress;
    updated_task.end_timestamp = String::new(); // Clear end timestamp

    store.save_task_execution(&updated_task).await?;

    // Check if all tasks are now running or complete
    let all_tasks = store.get_task_executions(execution_id).await?;
    let has_waiting_tasks = all_tasks
        .iter()
        .any(|t| t.status == crate::types::TaskStatus::WaitingForInput);

    info!(
        execution_id = %execution_id,
        task_count = all_tasks.len(),
        has_waiting_tasks = has_waiting_tasks,
        "Checking if workflow should be resumed"
    );

    if !has_waiting_tasks {
        // Mark workflow as resumed using new persistence method
        store.mark_workflow_resumed(execution_id).await?;

        info!(
            execution_id = %execution_id,
            "All tasks resumed, workflow status updated to Running"
        );
    } else {
        info!(
            execution_id = %execution_id,
            "Still has waiting tasks, workflow status remains WaitingForInput"
        );
    }

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Task resumed successfully"
    );

    Ok(())
}

/// Pause a workflow for user input
#[instrument(skip(execution_id, task_id), fields(execution_id, task_id))]
pub async fn pause_workflow(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Pausing workflow for user input"
    );

    // Load task execution
    let task_executions = store.get_task_executions(execution_id).await?;
    let task = task_executions
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| {
            CoreError::Validation(format!(
                "Task {} not found in execution {}",
                task_id, execution_id
            ))
        })?;

    // Update task status to waiting for input
    let mut updated_task = task.clone();
    updated_task.status = crate::types::TaskStatus::WaitingForInput;
    updated_task.end_timestamp = String::new(); // Clear end timestamp

    store.save_task_execution(&updated_task).await?;

    // Mark workflow as paused
    store.mark_workflow_paused(execution_id, task_id).await?;

    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Workflow paused successfully"
    );

    Ok(())
}
