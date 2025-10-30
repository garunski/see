use s_e_e_engine::{TaskInfo, TaskStatus as EngineTaskStatus};
use s_e_e_persistence::{TaskExecution, TaskExecutionStatus as PersistenceTaskExecutionStatus};
use std::collections::HashMap;

pub fn task_info_to_execution(
    task: &TaskInfo,
    workflow_id: &str,
    per_task_logs: &HashMap<String, Vec<String>>,
    errors: &[String],
    workflow_created_at: chrono::DateTime<chrono::Utc>,
    workflow_completed_at: chrono::DateTime<chrono::Utc>,
) -> TaskExecution {
    let output = per_task_logs
        .get(&task.id)
        .map(|logs| logs.join("\n"))
        .filter(|s| !s.is_empty());

    let persistence_status = match task.status {
        EngineTaskStatus::Pending => PersistenceTaskExecutionStatus::Pending,
        EngineTaskStatus::InProgress => PersistenceTaskExecutionStatus::InProgress,
        EngineTaskStatus::Complete => PersistenceTaskExecutionStatus::Complete,
        EngineTaskStatus::Failed => PersistenceTaskExecutionStatus::Failed,
        EngineTaskStatus::WaitingForInput => PersistenceTaskExecutionStatus::WaitingForInput,
    };

    let error = if matches!(task.status, EngineTaskStatus::Failed) {
        errors
            .iter()
            .find(|e| e.contains(&task.id))
            .cloned()
            .or_else(|| Some("Task failed".to_string()))
    } else {
        None
    };

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
        user_input: None,
        input_request_id: None,
        prompt_id: None,
    }
}

pub fn task_execution_to_info(task: &TaskExecution) -> TaskInfo {
    let engine_status = match task.status {
        PersistenceTaskExecutionStatus::Pending => EngineTaskStatus::Pending,
        PersistenceTaskExecutionStatus::InProgress => EngineTaskStatus::InProgress,
        PersistenceTaskExecutionStatus::Complete => EngineTaskStatus::Complete,
        PersistenceTaskExecutionStatus::Failed => EngineTaskStatus::Failed,
        PersistenceTaskExecutionStatus::WaitingForInput => EngineTaskStatus::WaitingForInput,
    };

    TaskInfo {
        id: task.id.clone(),
        name: task.name.clone(),
        status: engine_status,
    }
}
