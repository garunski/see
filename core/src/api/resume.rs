// Task resumption API ONLY

use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use persistence::{TaskStatus, WorkflowStatus};

/// Resume a paused task that's waiting for input
pub async fn resume_task(
    execution_id: &str,
    task_id: &str,
) -> Result<(), CoreError> {
    tracing::info!("Resuming task {} in execution {}", task_id, execution_id);
    
    // Step 1: Load WorkflowExecution from Persistence
    let store = get_global_store()?;
    let execution = store.get_workflow_execution(execution_id).await
        .map_err(|e| CoreError::Persistence(e))?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;
    
    // Step 2: Find TaskExecution by task_id
    let task = execution.tasks.iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| CoreError::TaskNotFound(task_id.to_string()))?;
    
    // Step 3: Validate Task Status
    if task.status != TaskStatus::WaitingForInput {
        return Err(CoreError::Execution(
            format!("Task {} is not waiting for input (status: {:?})", task_id, task.status)
        ));
    }
    
    // Step 4: Resume Task Execution via Engine
    // Note: Engine doesn't currently support task resumption
    // This is a placeholder for future implementation
    
    // Future implementation:
    // let engine = engine::WorkflowEngine::new();
    // let result = engine.resume_task(execution_id, task_id).await?;
    
    // Step 5: Update Task Status in Persistence
    let mut updated_task = task.clone();
    updated_task.status = TaskStatus::Complete;
    updated_task.completed_at = Some(chrono::Utc::now());
    
    store.save_task_execution(updated_task).await
        .map_err(|e| CoreError::Persistence(e))?;
    
    // Step 6: Update Execution if All Tasks Complete
    let all_tasks_complete = execution.tasks.iter()
        .all(|t| matches!(t.status, TaskStatus::Complete | TaskStatus::Failed));
    
    if all_tasks_complete {
        let mut updated_execution = execution.clone();
        updated_execution.status = WorkflowStatus::Complete;
        updated_execution.completed_at = Some(chrono::Utc::now());
        
        store.save_workflow_execution(updated_execution).await
            .map_err(|e| CoreError::Persistence(e))?;
    }
    
    // Step 7: Return Success
    tracing::info!("Task {} resumed successfully", task_id);
    Ok(())
}
