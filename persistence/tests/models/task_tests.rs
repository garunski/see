



use s_e_e_persistence::{TaskExecution, TaskStatus};
use chrono::Utc;

#[test]
fn test_task_execution_default() {
    let task = TaskExecution::default();

    assert!(!task.id.is_empty());
    assert!(task.workflow_id.is_empty());
    assert!(task.name.is_empty());
    assert_eq!(task.status, TaskExecutionStatus::Pending);
    assert!(task.output.is_none());
    assert!(task.error.is_none());
    assert!(task.created_at <= Utc::now());
    assert!(task.completed_at.is_none());
}

#[test]
fn test_task_execution_validation_success() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskExecutionStatus::Complete,
        output: Some("Task completed successfully".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };

    let result = task.validate();
    assert!(result.is_ok());
}

#[test]
fn test_task_execution_validation_empty_id() {
    let task = TaskExecution {
        id: "".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        ..Default::default()
    };

    let result = task.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ID cannot be empty"));
}

#[test]
fn test_task_execution_validation_empty_workflow_id() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "".to_string(),
        name: "Test Task".to_string(),
        ..Default::default()
    };

    let result = task.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Workflow ID cannot be empty"));
}

#[test]
fn test_task_execution_validation_empty_name() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "".to_string(),
        ..Default::default()
    };

    let result = task.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name cannot be empty"));
}

#[test]
fn test_task_execution_validation_complete_without_completion_time() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskExecutionStatus::Complete,
        completed_at: None,
        ..Default::default()
    };

    let result = task.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("completion timestamp"));
}

#[test]
fn test_task_execution_validation_waiting_with_completion_time() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskExecutionStatus::WaitingForInput,
        completed_at: Some(Utc::now()),
        ..Default::default()
    };

    let result = task.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("completion timestamp"));
}

#[test]
fn test_task_execution_is_finished() {
    let complete_task = TaskExecution {
        status: TaskExecutionStatus::Complete,
        ..Default::default()
    };
    assert!(complete_task.is_finished());

    let failed_task = TaskExecution {
        status: TaskExecutionStatus::Failed,
        ..Default::default()
    };
    assert!(failed_task.is_finished());

    let pending_task = TaskExecution {
        status: TaskExecutionStatus::Pending,
        ..Default::default()
    };
    assert!(!pending_task.is_finished());

    let waiting_task = TaskExecution {
        status: TaskExecutionStatus::WaitingForInput,
        ..Default::default()
    };
    assert!(!waiting_task.is_finished());
}

#[test]
fn test_task_execution_is_waiting_for_input() {
    let waiting_task = TaskExecution {
        status: TaskExecutionStatus::WaitingForInput,
        ..Default::default()
    };
    assert!(waiting_task.is_waiting_for_input());

    let pending_task = TaskExecution {
        status: TaskExecutionStatus::Pending,
        ..Default::default()
    };
    assert!(!pending_task.is_waiting_for_input());
}

#[test]
fn test_task_execution_serialization() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskExecutionStatus::Complete,
        output: Some("Task output".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };


    let json = serde_json::to_string(&task).unwrap();
    assert!(json.contains("task-1"));
    assert!(json.contains("workflow-1"));
    assert!(json.contains("Test Task"));
    assert!(json.contains("complete"));
    assert!(json.contains("Task output"));


    let deserialized: TaskExecution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, task.id);
    assert_eq!(deserialized.workflow_id, task.workflow_id);
    assert_eq!(deserialized.name, task.name);
    assert_eq!(deserialized.status, task.status);
    assert_eq!(deserialized.output, task.output);
}
