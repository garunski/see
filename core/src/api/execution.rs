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

pub async fn delete_workflow_execution(execution_id: &str) -> Result<(), CoreError> {
    tracing::info!("Deleting workflow execution: {}", execution_id);

    let store = get_global_store()?;

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

    store
        .delete_workflow_metadata_and_tasks(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    let pending_requests = store
        .get_pending_inputs_for_workflow(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    let mut deleted_count = 0;

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

    let all_requests = store
        .get_all_pending_inputs()
        .await
        .map_err(CoreError::Persistence)?;

    for request in all_requests {
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

    let execution_tasks = store
        .get_tasks_for_workflow(execution_id)
        .await
        .map_err(CoreError::Persistence)?;

    let task_ids: std::collections::HashSet<String> =
        execution_tasks.iter().map(|t| t.id.clone()).collect();

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

pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    tracing::debug!("Executing workflow: {}", workflow_id);

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

    tracing::debug!("Step 2: Validating workflow content");
    if workflow.content.is_empty() {
        return Err(CoreError::Execution(
            "Workflow content is empty".to_string(),
        ));
    }

    tracing::debug!("Step 3: Parsing workflow JSON");
    let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)
        .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;
    tracing::debug!("Step 3: Parsed workflow JSON successfully");

    tracing::debug!("Step 4: Converting to engine workflow");
    let engine_workflow = workflow_definition_to_engine(&workflow)?;
    tracing::debug!("Step 4: Converted to engine workflow");

    let execution_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let initial_execution = WorkflowExecution {
        id: execution_id.clone(),
        workflow_name: workflow.name.clone(),
        workflow_snapshot: workflow_json,
        status: WorkflowExecutionStatus::Running,
        created_at: now,
        completed_at: None,
        tasks: Vec::new(),
        timestamp: now,
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    tracing::debug!("Step 6: Saving initial execution to DB");
    store
        .save_workflow_execution(initial_execution.clone())
        .await
        .map_err(CoreError::Persistence)?;
    tracing::debug!("Step 6: Saved initial execution");

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

    let has_input_waiting = engine_result
        .tasks
        .iter()
        .any(|t| matches!(t.status, s_e_e_engine::TaskStatus::WaitingForInput));

    if has_input_waiting {
        tracing::debug!("Workflow paused - waiting for user input: {}", workflow_id);

        for task_info in &engine_result.tasks {
            if matches!(task_info.status, s_e_e_engine::TaskStatus::WaitingForInput) {
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

        let waiting_execution = workflow_result_to_execution(
            engine_result.clone(),
            execution_id.clone(),
            initial_execution.created_at,
        );

        let mut updated_execution = waiting_execution.clone();
        updated_execution.workflow_snapshot = initial_execution.workflow_snapshot;

        updated_execution.status = WorkflowExecutionStatus::WaitingForInput;
        updated_execution.completed_at = None;

        for task in &updated_execution.tasks {
            store
                .save_task_execution(task.clone())
                .await
                .map_err(CoreError::Persistence)?;
        }

        store
            .save_workflow_execution(updated_execution)
            .await
            .map_err(CoreError::Persistence)?;

        if let Some(ref callback) = callback {
            callback("Workflow paused - waiting for user input".to_string());
        }

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

    if let Some(ref callback) = callback {
        callback("Workflow execution completed".to_string());
    }

    let _completed_at = chrono::Utc::now();
    let mut final_execution = workflow_result_to_execution(
        engine_result.clone(),
        execution_id.clone(),
        initial_execution.created_at,
    );

    final_execution.workflow_snapshot = initial_execution.workflow_snapshot;

    for task in &final_execution.tasks {
        store
            .save_task_execution(task.clone())
            .await
            .map_err(CoreError::Persistence)?;
    }

    for audit_entry in &engine_result.audit_trail {
        let audit_event = audit_entry_to_event(audit_entry)?;
        store
            .log_audit_event(audit_event)
            .await
            .map_err(CoreError::Persistence)?;
    }

    store
        .save_workflow_execution(final_execution.clone())
        .await
        .map_err(CoreError::Persistence)?;

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

fn create_input_request_from_task(
    task_node: &Value,
    task_id: &str,
    execution_id: &str,
) -> Option<UserInputRequest> {
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
