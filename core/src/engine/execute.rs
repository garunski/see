use crate::{
    errors::CoreError,
    types::{
        AuditEntry as CoreAuditEntry, AuditStatus as CoreAuditStatus, OutputCallback,
        TaskInfo as CoreTaskInfo, TaskStatus as CoreTaskStatus,
        WorkflowResult as CoreWorkflowResult,
    },
};
use engine::{
    execute_workflow_from_json, AuditEntry as EngineAuditEntry, AuditStatus as EngineAuditStatus,
    TaskInfo as EngineTaskInfo, TaskStatus as EngineTaskStatus,
    WorkflowResult as EngineWorkflowResult,
};
use serde_json::Value;
use tokio::fs;
use tracing::{debug, info, instrument};

/// Convert engine TaskStatus to core TaskStatus
fn convert_task_status(status: EngineTaskStatus) -> CoreTaskStatus {
    match status {
        EngineTaskStatus::Pending => CoreTaskStatus::Pending,
        EngineTaskStatus::InProgress => CoreTaskStatus::InProgress,
        EngineTaskStatus::Complete => CoreTaskStatus::Complete,
        EngineTaskStatus::Failed => CoreTaskStatus::Failed,
        EngineTaskStatus::WaitingForInput => CoreTaskStatus::WaitingForInput,
    }
}

/// Convert engine AuditStatus to core AuditStatus
fn convert_audit_status(status: EngineAuditStatus) -> CoreAuditStatus {
    match status {
        EngineAuditStatus::Success => CoreAuditStatus::Success,
        EngineAuditStatus::Failure => CoreAuditStatus::Failure,
    }
}

/// Convert engine TaskInfo to core TaskInfo
fn convert_task_info(task: EngineTaskInfo) -> CoreTaskInfo {
    CoreTaskInfo {
        id: task.id,
        name: task.name,
        status: convert_task_status(task.status),
    }
}

/// Convert engine AuditEntry to core AuditEntry
fn convert_audit_entry(entry: EngineAuditEntry) -> CoreAuditEntry {
    CoreAuditEntry {
        task_id: entry.task_id,
        status: convert_audit_status(entry.status),
        timestamp: entry.timestamp,
        changes_count: entry.changes_count,
        message: entry.message,
    }
}

/// Convert engine WorkflowResult to core WorkflowResult
fn convert_workflow_result(engine_result: EngineWorkflowResult) -> CoreWorkflowResult {
    CoreWorkflowResult {
        success: engine_result.success,
        workflow_name: engine_result.workflow_name,
        task_count: engine_result.tasks.len(),
        execution_id: uuid::Uuid::new_v4().to_string(), // Generate new execution ID
        tasks: engine_result
            .tasks
            .into_iter()
            .map(convert_task_info)
            .collect(),
        final_context: Value::Object(serde_json::Map::new()), // Empty context for now
        audit_trail: engine_result
                .audit_trail
            .into_iter()
            .map(convert_audit_entry)
            .collect(),
        per_task_logs: engine_result.per_task_logs,
        errors: engine_result.errors,
        output_logs: Vec::new(), // Empty for now
    }
}

#[instrument(skip(workflow_data, _output_callback), fields(execution_id))]
pub async fn execute_workflow_from_content(
    workflow_data: &str,
    _output_callback: Option<OutputCallback>,
) -> Result<CoreWorkflowResult, CoreError> {
    info!("Using new workflow engine");

    // Execute workflow using the new engine
    let engine_result = execute_workflow_from_json(workflow_data)
        .await
        .map_err(|e| CoreError::WorkflowExecution(format!("Workflow execution failed: {}", e)))?;

    // Convert to core format
    let core_result = convert_workflow_result(engine_result);

    // TODO: Add persistence later
    info!("Workflow execution completed with ID: {}", core_result.execution_id);

    Ok(core_result)
}

#[instrument(skip(_output_callback), fields(workflow_file = %workflow_file))]
pub async fn execute_workflow(
    workflow_file: &str,
    _output_callback: Option<OutputCallback>,
) -> Result<CoreWorkflowResult, CoreError> {
    debug!("Reading workflow file");
    let workflow_data = fs::read_to_string(workflow_file).await.map_err(|e| {
        CoreError::WorkflowExecution(format!(
            "Failed to read workflow file '{}': {}",
            workflow_file, e
        ))
    })?;

    execute_workflow_from_content(&workflow_data, _output_callback).await
}

#[instrument(skip(_output_callback), fields(workflow_id = %workflow_id))]
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    _output_callback: Option<OutputCallback>,
) -> Result<CoreWorkflowResult, CoreError> {
    debug!("Fetching workflow definition by ID");
    // TODO: Add store back later
    // let workflow_definition = store.get_workflow_definition(workflow_id).await?;

    debug!(
        workflow_id = %workflow_id,
        workflow_name = "TODO",
        "Found workflow definition, executing"
    );

    // TODO: Add workflow definition back
    Err(CoreError::Validation("Workflow definition not available".to_string()))
}

/// Resume a workflow that is waiting for user input
#[instrument(skip(execution_id))]
pub async fn resume_workflow(execution_id: &str) -> Result<(), CoreError> {
    info!("Resuming workflow execution: {}", execution_id);
    
    // TODO: Implement workflow resumption with new persistence layer
    Err(CoreError::Validation("Workflow resumption not yet implemented".to_string()))
}

/// Resume a specific task that is waiting for user input
#[instrument(skip(execution_id, task_id))]
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    info!("Resuming task execution: {} in workflow {}", task_id, execution_id);
    
    // TODO: Implement task resumption with new persistence layer
    Err(CoreError::Validation("Task resumption not yet implemented".to_string()))
}

/// Pause a workflow for user input
#[instrument(skip(execution_id, task_id))]
pub async fn pause_workflow(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    info!("Pausing workflow execution: {} for task {}", execution_id, task_id);
    
    // TODO: Implement workflow pausing with new persistence layer
    Err(CoreError::Validation("Workflow pausing not yet implemented".to_string()))
}