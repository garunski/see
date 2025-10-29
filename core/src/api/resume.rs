// Task resumption API ONLY

use crate::bridge::audit::audit_entry_to_event;
use crate::bridge::execution::workflow_result_to_execution;
use crate::bridge::{OutputCallback, WorkflowResult};
use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use s_e_e_engine::WorkflowEngine;
use s_e_e_persistence::{TaskExecutionStatus, WorkflowExecutionStatus};
use std::collections::{HashMap, HashSet};

/// Resume workflow execution after user input has been provided
/// This continues the workflow from where it left off
pub async fn resume_workflow_execution(
    execution_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    tracing::info!("Resuming workflow execution: {}", execution_id);

    // Step 1: Load WorkflowExecution from Persistence
    let store = get_global_store()?;

    // Load execution metadata (without tasks first)
    let mut execution = store
        .get_workflow_execution(execution_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;

    // Always reload tasks from the database to ensure we have the latest state
    // This is critical because tasks are updated separately via save_task_execution
    let fresh_tasks = store
        .get_tasks_for_workflow(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    execution.tasks = fresh_tasks;

    tracing::debug!(
        execution_id = %execution_id,
        status = ?execution.status,
        task_count = execution.tasks.len(),
        "Loaded workflow execution with fresh tasks from database"
    );

    // Step 2: Validate execution is in a state where it can be resumed
    if !matches!(
        execution.status,
        WorkflowExecutionStatus::WaitingForInput | WorkflowExecutionStatus::Running
    ) {
        return Err(CoreError::Execution(format!(
            "Workflow execution {} cannot be resumed from status: {:?}",
            execution_id, execution.status
        )));
    }

    // Step 3: Parse workflow snapshot to EngineWorkflow
    let workflow_json_str = serde_json::to_string(&execution.workflow_snapshot).map_err(|e| {
        CoreError::Execution(format!("Failed to serialize workflow snapshot: {}", e))
    })?;

    let engine_workflow = s_e_e_engine::parse_workflow(&workflow_json_str)
        .map_err(|e| CoreError::Engine(s_e_e_engine::EngineError::Parser(e)))?;

    tracing::debug!(
        execution_id = %execution_id,
        workflow_name = %engine_workflow.name,
        task_count = engine_workflow.tasks.len(),
        "Parsed workflow from snapshot"
    );

    // Step 4: Build set of completed tasks and user inputs
    // IMPORTANT: We iterate through tasks from the database to ensure we have the latest state
    let mut completed_task_ids = HashSet::new();
    let mut task_user_inputs = HashMap::new();

    tracing::debug!(
        execution_id = %execution_id,
        task_count = execution.tasks.len(),
        "Analyzing {} tasks to build execution state",
        execution.tasks.len()
    );

    for task_execution in &execution.tasks {
        tracing::trace!(
            execution_id = %execution_id,
            task_id = %task_execution.id,
            task_name = %task_execution.name,
            status = ?task_execution.status,
            has_user_input = task_execution.user_input.is_some(),
            "Analyzing task"
        );

        match task_execution.status {
            TaskExecutionStatus::Complete | TaskExecutionStatus::Failed => {
                completed_task_ids.insert(task_execution.id.clone());
            }
            TaskExecutionStatus::WaitingForInput => {
                // If task has user_input, it means input was provided but workflow hasn't resumed yet
                if let Some(ref user_input) = task_execution.user_input {
                    tracing::debug!(
                        execution_id = %execution_id,
                        task_id = %task_execution.id,
                        "Found task with user input provided"
                    );
                    completed_task_ids.insert(task_execution.id.clone());
                    task_user_inputs.insert(task_execution.id.clone(), user_input.clone());
                }
            }
            _ => {
                // Tasks that are still pending or in progress
            }
        }
    }

    tracing::debug!(
        execution_id = %execution_id,
        completed_count = completed_task_ids.len(),
        user_input_count = task_user_inputs.len(),
        "Built execution state"
    );

    // Step 5: Resume workflow execution via Engine
    let engine = WorkflowEngine::new();
    let engine_result = engine
        .resume_workflow_execution(
            engine_workflow,
            execution_id.to_string(),
            completed_task_ids,
            task_user_inputs,
        )
        .await
        .map_err(CoreError::Engine)?;

    tracing::debug!(
        execution_id = %execution_id,
        success = engine_result.success,
        task_count = engine_result.tasks.len(),
        "Engine resume completed"
    );

    // Step 6: Check if workflow is waiting for input again
    let has_input_waiting = engine_result
        .tasks
        .iter()
        .any(|t| matches!(t.status, s_e_e_engine::TaskStatus::WaitingForInput));

    if has_input_waiting {
        tracing::debug!(
            execution_id = %execution_id,
            "Workflow paused - waiting for user input again"
        );

        // Create UserInputRequest records for tasks waiting for input
        for task_info in &engine_result.tasks {
            if matches!(task_info.status, s_e_e_engine::TaskStatus::WaitingForInput) {
                if let Some(task_node) =
                    find_task_in_snapshot(&execution.workflow_snapshot, &task_info.id)
                {
                    if let Some(input_request) =
                        create_input_request_from_task(task_node, &task_info.id, execution_id)
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

        // Convert Engine Result to Persistence Types
        let waiting_execution = workflow_result_to_execution(
            engine_result.clone(),
            execution_id.to_string(),
            execution.created_at,
        );

        // Preserve the workflow snapshot
        let mut updated_execution = waiting_execution.clone();
        updated_execution.workflow_snapshot = execution.workflow_snapshot;
        updated_execution.status = WorkflowExecutionStatus::WaitingForInput;
        updated_execution.completed_at = None;

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
            execution_id: execution_id.to_string(),
            tasks: engine_result.tasks,
            audit_trail: engine_result.audit_trail,
            per_task_logs: engine_result.per_task_logs,
            errors: vec!["Waiting for user input".to_string()],
        });
    }

    // Step 7: Workflow completed - save final state
    let _completed_at = chrono::Utc::now();
    let mut final_execution = workflow_result_to_execution(
        engine_result.clone(),
        execution_id.to_string(),
        execution.created_at,
    );

    // Preserve the workflow snapshot
    final_execution.workflow_snapshot = execution.workflow_snapshot;

    // Save Task Executions to Persistence
    for task in &final_execution.tasks {
        store
            .save_task_execution(task.clone())
            .await
            .map_err(CoreError::Persistence)?;
    }

    // Save Audit Events to Persistence
    for audit_entry in &engine_result.audit_trail {
        let audit_event = audit_entry_to_event(audit_entry)?;
        store
            .log_audit_event(audit_event)
            .await
            .map_err(CoreError::Persistence)?;
    }

    // Update Final Execution Record
    store
        .save_workflow_execution(final_execution)
        .await
        .map_err(CoreError::Persistence)?;

    // Stream progress via OutputCallback
    if let Some(ref callback) = callback {
        callback("Workflow execution completed".to_string());
    }

    // Step 8: Return WorkflowResult
    let result = WorkflowResult {
        success: engine_result.success,
        workflow_name: engine_result.workflow_name,
        execution_id: execution_id.to_string(),
        tasks: engine_result.tasks,
        audit_trail: engine_result.audit_trail,
        per_task_logs: engine_result.per_task_logs,
        errors: engine_result.errors,
    };

    tracing::info!(
        execution_id = %execution_id,
        success = result.success,
        "Workflow resume execution completed"
    );

    Ok(result)
}

/// Find a task in the workflow snapshot by task ID
fn find_task_in_snapshot<'a>(
    snapshot: &'a serde_json::Value,
    task_id: &str,
) -> Option<&'a serde_json::Value> {
    if let Some(tasks) = snapshot.get("tasks").and_then(|t| t.as_array()) {
        fn search_task<'a>(
            tasks: &'a [serde_json::Value],
            task_id: &str,
        ) -> Option<&'a serde_json::Value> {
            for task in tasks {
                if let Some(id) = task.get("id").and_then(|v| v.as_str()) {
                    if id == task_id {
                        return Some(task);
                    }
                }
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
    task_node: &serde_json::Value,
    task_id: &str,
    execution_id: &str,
) -> Option<s_e_e_persistence::UserInputRequest> {
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

    let input_type = match input_type_str {
        "number" => s_e_e_persistence::InputType::Number,
        "boolean" => s_e_e_persistence::InputType::Boolean,
        _ => s_e_e_persistence::InputType::String,
    };

    Some(s_e_e_persistence::UserInputRequest {
        id: uuid::Uuid::new_v4().to_string(),
        task_execution_id: task_id.to_string(),
        workflow_execution_id: execution_id.to_string(),
        prompt_text: prompt,
        input_type,
        required,
        default_value: default,
        validation_rules: serde_json::Value::Object(serde_json::Map::new()),
        status: s_e_e_persistence::InputRequestStatus::Pending,
        created_at: chrono::Utc::now(),
        fulfilled_at: None,
        fulfilled_value: None,
    })
}
