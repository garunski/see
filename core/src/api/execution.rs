// Workflow execution API ONLY

use crate::bridge::audit::audit_entry_to_event;
use crate::bridge::execution::workflow_result_to_execution;
use crate::bridge::workflow::workflow_definition_to_engine;
use crate::bridge::{OutputCallback, WorkflowResult};
use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use engine::WorkflowEngine;
use persistence::{WorkflowExecution, WorkflowStatus};

/// Execute a workflow by loading it from persistence and running it through the engine
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    tracing::debug!("Executing workflow: {}", workflow_id);

    // Step 1: Load WorkflowDefinition from Persistence
    let store = get_global_store()?;
    let workflow = store
        .get_workflow(workflow_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(workflow_id.to_string()))?;

    // Step 2: Validate Workflow Content
    if workflow.content.is_empty() {
        return Err(CoreError::Execution(
            "Workflow content is empty".to_string(),
        ));
    }

    // Step 3: Parse workflow content to JSON for snapshot
    let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)
        .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;

    // Step 4: Parse JSON Content to EngineWorkflow
    let engine_workflow = workflow_definition_to_engine(&workflow)?;

    // Step 5: Create Initial WorkflowExecution Record
    let execution_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let initial_execution = WorkflowExecution {
        id: execution_id.clone(),
        workflow_name: workflow.name.clone(),
        workflow_snapshot: workflow_json,
        status: WorkflowStatus::Running,
        created_at: now,
        completed_at: None,
        success: None,
        tasks: Vec::new(), // Will be populated after execution
        timestamp: now,
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    // Step 6: Save Initial Execution to Persistence
    store
        .save_workflow_execution(initial_execution.clone())
        .await
        .map_err(CoreError::Persistence)?;

    // Step 7: Execute Workflow Through Engine
    let engine = WorkflowEngine::new();
    let engine_result = engine
        .execute_workflow(engine_workflow)
        .await
        .map_err(CoreError::Engine)?;

    // Step 8: Check if workflow is waiting for input
    let has_input_waiting = engine_result
        .tasks
        .iter()
        .any(|t| matches!(t.status, engine::TaskStatus::WaitingForInput));

    if has_input_waiting {
        tracing::debug!("Workflow paused - waiting for user input: {}", workflow_id);

        // Convert Engine Result to Persistence Types BEFORE saving
        let waiting_execution = workflow_result_to_execution(
            engine_result.clone(),
            execution_id.clone(),
            initial_execution.created_at,
        );

        // Preserve the workflow snapshot for task ordering
        let mut updated_execution = waiting_execution.clone();
        updated_execution.workflow_snapshot = initial_execution.workflow_snapshot;

        // Update status to Running (waiting for input)
        updated_execution.status = WorkflowStatus::Running;
        updated_execution.completed_at = None; // Not completed yet
        updated_execution.success = None; // Unknown until input provided

        // Save Task Executions to Persistence
        for task in &updated_execution.tasks {
            store
                .save_task_execution(task.clone())
                .await
                .map_err(CoreError::Persistence)?;
        }

        // Save the execution with tasks
        store
            .save_workflow_execution(updated_execution)
            .await
            .map_err(CoreError::Persistence)?;

        // Stream progress via OutputCallback
        if let Some(ref callback) = callback {
            callback("Workflow paused - waiting for user input".to_string());
        }

        // Return special status indicating waiting for input
        return Ok(WorkflowResult {
            success: false,
            workflow_name: engine_result.workflow_name,
            execution_id,
            tasks: engine_result.tasks,
            audit_trail: engine_result.audit_trail,
            per_task_logs: engine_result.per_task_logs,
            errors: vec!["Waiting for user input".to_string()],
        });
    }

    // Step 9: Stream Progress via OutputCallback
    if let Some(ref callback) = callback {
        callback("Workflow execution completed".to_string());
    }

    // Step 10: Convert Engine Result to Persistence Types
    let _completed_at = chrono::Utc::now();
    let mut final_execution = workflow_result_to_execution(
        engine_result.clone(),
        execution_id.clone(),
        initial_execution.created_at,
    );

    // Preserve the workflow snapshot for task ordering
    final_execution.workflow_snapshot = initial_execution.workflow_snapshot;

    // Step 11: Save Task Executions to Persistence
    for task in &final_execution.tasks {
        store
            .save_task_execution(task.clone())
            .await
            .map_err(CoreError::Persistence)?;
    }

    // Step 12: Save Audit Events to Persistence
    for audit_entry in &engine_result.audit_trail {
        let audit_event = audit_entry_to_event(audit_entry)?;
        store
            .log_audit_event(audit_event)
            .await
            .map_err(CoreError::Persistence)?;
    }

    // Step 13: Update Final Execution Record
    store
        .save_workflow_execution(final_execution.clone())
        .await
        .map_err(CoreError::Persistence)?;

    // Step 14: Return WorkflowResult
    let result = WorkflowResult {
        success: engine_result.success,
        workflow_name: engine_result.workflow_name,
        execution_id,
        tasks: engine_result.tasks,
        audit_trail: engine_result.audit_trail,
        per_task_logs: engine_result.per_task_logs,
        errors: engine_result.errors,
    };

    tracing::info!(
        "Workflow execution completed: {} (execution_id: {})",
        result.success,
        result.execution_id
    );
    Ok(result)
}
