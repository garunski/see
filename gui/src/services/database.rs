use tracing::{debug, error, info};

pub async fn clear_database() -> Result<(), String> {
    debug!("Clearing database");
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    store
        .clear_all_data()
        .await
        .map_err(|e| format!("Failed to clear database: {}", e))?;

    info!("Database cleared successfully");
    Ok(())
}

pub async fn get_workflow_executions() -> Result<Vec<s_e_e_core::WorkflowExecution>, String> {
    debug!("Getting workflow executions from database");
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    let workflows = store
        .list_workflow_executions()
        .await
        .map_err(|e| format!("Failed to get workflow executions: {}", e))?;

    info!("Retrieved {} workflow executions", workflows.len());
    Ok(workflows)
}

pub async fn get_tasks_for_workflow(
    workflow_id: &str,
) -> Result<Vec<s_e_e_core::TaskExecution>, String> {
    debug!("Getting tasks for workflow: {}", workflow_id);
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    let tasks = store
        .get_tasks_for_workflow(workflow_id)
        .await
        .map_err(|e| format!("Failed to get tasks for workflow: {}", e))?;

    info!(
        "Retrieved {} tasks for workflow: {}",
        tasks.len(),
        workflow_id
    );
    Ok(tasks)
}

pub async fn save_workflow_execution(
    workflow: s_e_e_core::WorkflowExecution,
) -> Result<(), String> {
    debug!("Saving workflow execution: {}", workflow.id);
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    store
        .save_workflow_execution(workflow)
        .await
        .map_err(|e| format!("Failed to save workflow execution: {}", e))?;

    info!("Workflow execution saved successfully");
    Ok(())
}

pub async fn save_task_execution(task: s_e_e_core::TaskExecution) -> Result<(), String> {
    debug!("Saving task execution: {}", task.id);
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    store
        .save_task_execution(task)
        .await
        .map_err(|e| format!("Failed to save task execution: {}", e))?;

    info!("Task execution saved successfully");
    Ok(())
}

pub async fn log_audit_event(event: s_e_e_core::AuditEvent) -> Result<(), String> {
    debug!("Logging audit event: {}", event.id);
    let store =
        s_e_e_core::get_global_store().map_err(|e| format!("Database unavailable: {}", e))?;

    store
        .log_audit_event(event)
        .await
        .map_err(|e| format!("Failed to log audit event: {}", e))?;

    debug!("Audit event logged successfully");
    Ok(())
}
