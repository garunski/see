// Task conversions ONLY

use engine::TaskInfo;
use persistence::{TaskExecution, TaskStatus as PersistenceTaskStatus};
use engine::TaskStatus as EngineTaskStatus;
use std::collections::HashMap;

/// Convert TaskInfo to TaskExecution
pub fn task_info_to_execution(
    task: &TaskInfo,
    workflow_id: &str,
    per_task_logs: &HashMap<String, Vec<String>>,
    errors: &Vec<String>,
    workflow_created_at: chrono::DateTime<chrono::Utc>,
    workflow_completed_at: chrono::DateTime<chrono::Utc>,
) -> TaskExecution {
    // Extract output from per_task_logs
    let output = per_task_logs.get(&task.id)
        .map(|logs| logs.join("\n"))
        .filter(|s| !s.is_empty());
    
    // Convert engine TaskStatus to persistence TaskStatus
    let persistence_status = match task.status {
        EngineTaskStatus::Pending => PersistenceTaskStatus::Pending,
        EngineTaskStatus::InProgress => PersistenceTaskStatus::InProgress,
        EngineTaskStatus::Complete => PersistenceTaskStatus::Complete,
        EngineTaskStatus::Failed => PersistenceTaskStatus::Failed,
        EngineTaskStatus::WaitingForInput => PersistenceTaskStatus::WaitingForInput,
    };
    
    // Extract error if task failed
    let error = if matches!(task.status, EngineTaskStatus::Failed) {
        // Try to find task-specific error in errors vec
        errors.iter()
            .find(|e| e.contains(&task.id))
            .cloned()
            .or_else(|| Some("Task failed".to_string()))
    } else {
        None
    };
    
    // Estimate timestamps (engine doesn't provide per-task timestamps)
    let completed_at = match task.status {
        EngineTaskStatus::Complete | EngineTaskStatus::Failed => Some(workflow_completed_at),
        EngineTaskStatus::WaitingForInput => None,
        _ => None,
    };
    
    TaskExecution {
        id: task.id.clone(),
        workflow_id: workflow_id.to_string(),
        name: task.name.clone(),
        status: persistence_status,
        output,
        error,
        created_at: workflow_created_at,
        completed_at,
    }
}
