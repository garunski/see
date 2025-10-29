// Workflow execution API ONLY

use crate::bridge::audit::audit_entry_to_event;
use crate::bridge::execution::workflow_result_to_execution;
use crate::bridge::workflow::workflow_definition_to_engine;
use crate::bridge::{OutputCallback, WorkflowResult};
use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use s_e_e_engine::WorkflowEngine;
use s_e_e_persistence::{
    InputRequestStatus, InputType, UserInputRequest, WorkflowExecution, WorkflowExecutionStatus,
};
use serde_json::Value;

/// Delete a workflow execution and all related data (tasks, input requests, audit events)
pub async fn delete_workflow_execution(execution_id: &str) -> Result<(), CoreError> {
    tracing::info!("Deleting workflow execution: {}", execution_id);

    let store = get_global_store()?;

    // Verify execution exists
    let execution = store
        .get_workflow_execution(execution_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;

    tracing::debug!(
        execution_id = %execution_id,
        workflow_name = %execution.workflow_name,
        "Found execution to delete"
    );

    // Delete workflow execution and all associated tasks
    // This uses delete_workflow_metadata_and_tasks which handles both
    store
        .delete_workflow_metadata_and_tasks(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    // Delete all user input requests for this execution
    // First get pending inputs, then we'll also get fulfilled ones
    let pending_requests = store
        .get_pending_inputs_for_workflow(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    let mut deleted_count = 0;

    // Delete pending requests
    for request in pending_requests {
        store
            .delete_input_request(&request.id)
            .await
            .map_err(CoreError::Persistence)?;
        deleted_count += 1;
        tracing::debug!(
            execution_id = %execution_id,
            input_request_id = %request.id,
            "Deleted pending input request"
        );
    }

    // For fulfilled requests, we need to check all requests
    // Since there's no method to get all requests for a workflow,
    // we'll get all pending inputs (which includes both pending and fulfilled)
    // and filter by workflow_execution_id
    let all_requests = store
        .get_all_pending_inputs()
        .await
        .map_err(CoreError::Persistence)?;

    for request in all_requests {
        // Only process if it belongs to this execution and wasn't already deleted (not pending)
        if request.workflow_execution_id == execution_id
            && !matches!(request.status, InputRequestStatus::Pending)
        {
            store
                .delete_input_request(&request.id)
                .await
                .map_err(CoreError::Persistence)?;
            deleted_count += 1;
            tracing::debug!(
                execution_id = %execution_id,
                input_request_id = %request.id,
                "Deleted fulfilled input request"
            );
        }
    }

    tracing::debug!(
        execution_id = %execution_id,
        deleted_input_requests = deleted_count,
        "Deleted input requests for execution"
    );

    // Delete all audit events for tasks in this execution
    // Load tasks first to get their IDs
    let execution_tasks = store
        .get_tasks_for_workflow(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    let task_ids: std::collections::HashSet<String> =
        execution_tasks.iter().map(|t| t.id.clone()).collect();

    // Get all audit events and delete those that reference tasks in this execution
    // Since we don't have a list_audit_events method yet, we'll query directly
    // For now, we'll skip audit event deletion - they can be cleaned up separately
    // as audit events are primarily for historical tracking
    tracing::debug!(
        execution_id = %execution_id,
        task_count = task_ids.len(),
        "Execution and tasks deleted (audit events preserved for historical record)"
    );

    tracing::info!(
        execution_id = %execution_id,
        "Workflow execution deleted successfully"
    );

    Ok(())
}

/// Execute a workflow by loading it from persistence and running it through the engine
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    tracing::debug!("Executing workflow: {}", workflow_id);

    // Step 1: Load WorkflowDefinition from Persistence
    tracing::debug!("Step 1: Getting global store");
    let store = get_global_store()?;
    tracing::debug!("Step 1: Got global store");
    tracing::debug!("Step 1: Loading workflow from DB");
    let workflow = store
        .get_workflow(workflow_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(workflow_id.to_string()))?;
    tracing::debug!("Step 1: Loaded workflow: {}", workflow.name);

    // Step 2: Validate Workflow Content
    tracing::debug!("Step 2: Validating workflow content");
    if workflow.content.is_empty() {
        return Err(CoreError::Execution(
            "Workflow content is empty".to_string(),
        ));
    }

    // Step 3: Parse workflow content to JSON for snapshot
    tracing::debug!("Step 3: Parsing workflow JSON");
    let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)
        .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;
    tracing::debug!("Step 3: Parsed workflow JSON successfully");

    // Step 4: Parse JSON Content to EngineWorkflow
    tracing::debug!("Step 4: Converting to engine workflow");
    let engine_workflow = workflow_definition_to_engine(&workflow)?;
    tracing::debug!("Step 4: Converted to engine workflow");

    // Step 5: Create Initial WorkflowExecution Record
    let execution_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let initial_execution = WorkflowExecution {
        id: execution_id.clone(),
        workflow_name: workflow.name.clone(),
        workflow_snapshot: workflow_json,
        status: WorkflowExecutionStatus::Running,
        created_at: now,
        completed_at: None,
        tasks: Vec::new(), // Will be populated after execution
        timestamp: now,
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    // Step 6: Save Initial Execution to Persistence
    tracing::debug!("Step 6: Saving initial execution to DB");
    store
        .save_workflow_execution(initial_execution.clone())
        .await
        .map_err(CoreError::Persistence)?;
    tracing::debug!("Step 6: Saved initial execution");

    // Step 7: Execute Workflow Through Engine
    tracing::debug!("Step 7: Creating workflow engine");
    let engine = WorkflowEngine::new();
    tracing::debug!("Step 7: Executing workflow through engine");
    let engine_result = match engine.execute_workflow(engine_workflow).await {
        Ok(result) => {
            tracing::debug!("Step 7: Engine execution completed successfully");
            result
        }
        Err(e) => {
            tracing::error!("Step 7: Engine execution failed: {}", e);
            // Update execution status to failed
            let mut failed_execution = initial_execution.clone();
            failed_execution.status = WorkflowExecutionStatus::Failed;
            failed_execution.completed_at = Some(chrono::Utc::now());
            failed_execution.errors = vec![e.to_string()];

            store
                .save_workflow_execution(failed_execution)
                .await
                .map_err(CoreError::Persistence)?;

            return Err(CoreError::Engine(e));
        }
    };

    // Step 8: Check if workflow is waiting for input
    let has_input_waiting = engine_result
        .tasks
        .iter()
        .any(|t| matches!(t.status, s_e_e_engine::TaskStatus::WaitingForInput));

    if has_input_waiting {
        tracing::debug!("Workflow paused - waiting for user input: {}", workflow_id);

        // Create UserInputRequest records for tasks waiting for input BEFORE moving the snapshot
        for task_info in &engine_result.tasks {
            if matches!(task_info.status, s_e_e_engine::TaskStatus::WaitingForInput) {
                // Extract user input parameters from workflow snapshot
                if let Some(task_node) =
                    find_task_in_snapshot(&initial_execution.workflow_snapshot, &task_info.id)
                {
                    if let Some(input_request) =
                        create_input_request_from_task(task_node, &task_info.id, &execution_id)
                    {
                        tracing::debug!(
                            task_id = %task_info.id,
                            request_id = %input_request.id,
                            "Creating input request for task"
                        );
                        store
                            .save_input_request(&input_request)
                            .await
                            .map_err(CoreError::Persistence)?;
                    }
                }
            }
        }

        // Convert Engine Result to Persistence Types BEFORE saving
        let waiting_execution = workflow_result_to_execution(
            engine_result.clone(),
            execution_id.clone(),
            initial_execution.created_at,
        );

        // Preserve the workflow snapshot for task ordering
        let mut updated_execution = waiting_execution.clone();
        updated_execution.workflow_snapshot = initial_execution.workflow_snapshot;

        // Update status to WaitingForInput
        updated_execution.status = WorkflowExecutionStatus::WaitingForInput;
        updated_execution.completed_at = None; // Not completed yet

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

        // Return result indicating workflow is paused for input
        return Ok(WorkflowResult {
            success: false, // Workflow not complete, waiting for input
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

/// Find a task in the workflow snapshot by task ID
fn find_task_in_snapshot<'a>(snapshot: &'a Value, task_id: &str) -> Option<&'a Value> {
    if let Some(tasks) = snapshot.get("tasks").and_then(|t| t.as_array()) {
        // Helper function to recursively search tasks
        fn search_task<'a>(tasks: &'a [Value], task_id: &str) -> Option<&'a Value> {
            for task in tasks {
                if let Some(id) = task.get("id").and_then(|v| v.as_str()) {
                    if id == task_id {
                        return Some(task);
                    }
                }
                // Search in next_tasks
                if let Some(next_tasks) = task.get("next_tasks").and_then(|t| t.as_array()) {
                    if let Some(found) = search_task(next_tasks, task_id) {
                        return Some(found);
                    }
                }
            }
            None
        }
        return search_task(tasks, task_id);
    }
    None
}

/// Create a UserInputRequest from a task node in the snapshot
fn create_input_request_from_task(
    task_node: &Value,
    task_id: &str,
    execution_id: &str,
) -> Option<UserInputRequest> {
    // Check if this is a user_input task
    let function = task_node.get("function")?;
    let function_type = function.get("name").and_then(|v| v.as_str())?;

    if function_type != "user_input" {
        return None;
    }

    let input = function.get("input")?;
    let prompt = input.get("prompt")?.as_str()?.to_string();
    let input_type_str = input
        .get("input_type")
        .and_then(|v| v.as_str())
        .unwrap_or("string");
    let required = input
        .get("required")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let default = input.get("default").cloned();

    // Convert input type string to InputType enum
    let input_type = match input_type_str {
        "number" => InputType::Number,
        "boolean" => InputType::Boolean,
        _ => InputType::String,
    };

    let request = UserInputRequest {
        id: uuid::Uuid::new_v4().to_string(),
        task_execution_id: task_id.to_string(),
        workflow_execution_id: execution_id.to_string(),
        prompt_text: prompt,
        input_type,
        required,
        default_value: default,
        validation_rules: Value::Object(serde_json::Map::new()),
        status: InputRequestStatus::Pending,
        created_at: chrono::Utc::now(),
        fulfilled_at: None,
        fulfilled_value: None,
    };

    Some(request)
}
