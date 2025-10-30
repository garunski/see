



use s_e_e_persistence::{WorkflowExecutionStatus, TaskExecution, TaskStatus, WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata};
use chrono::Utc;

#[test]
fn test_workflow_execution_default() {
    let execution = WorkflowExecution::default();

    assert!(!execution.id.is_empty());
    assert!(execution.workflow_name.is_empty());
    assert_eq!(execution.workflow_snapshot, serde_json::json!({}));
    assert_eq!(execution.status, WorkflowExecutionStatus::Pending);
    assert!(execution.created_at <= Utc::now());
    assert!(execution.completed_at.is_none());
    assert!(execution.tasks.is_empty());
    assert!(execution.timestamp <= Utc::now());
}

#[test]
fn test_workflow_execution_serialization() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test Workflow",
            "tasks": []
        }),
        status: WorkflowExecutionStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };


    let json = serde_json::to_string(&execution).unwrap();
    assert!(json.contains("exec-1"));
    assert!(json.contains("Test Workflow"));
    assert!(json.contains("complete"));


    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, execution.id);
    assert_eq!(deserialized.workflow_name, execution.workflow_name);
    assert_eq!(deserialized.status, execution.status);
}

#[test]
fn test_workflow_execution_serialization_with_snapshot() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test Workflow",
            "tasks": [
                {"id": "task1", "name": "Task 1"},
                {"id": "task2", "name": "Task 2"}
            ]
        }),
        status: WorkflowExecutionStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    let json = serde_json::to_string(&execution).unwrap();
    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.workflow_snapshot, execution.workflow_snapshot);
    assert_eq!(deserialized.id, execution.id);
    assert_eq!(deserialized.workflow_name, execution.workflow_name);
}

#[test]
fn test_workflow_execution_to_summary() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test Workflow",
            "tasks": []
        }),
        status: WorkflowExecutionStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        tasks: vec![
            TaskExecution::default(),
            TaskExecution::default(),
        ],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    let summary = execution.to_summary();

    assert_eq!(summary.id, execution.id);
    assert_eq!(summary.workflow_name, execution.workflow_name);
    assert_eq!(summary.status, execution.status);
    assert_eq!(summary.created_at, execution.created_at);
    assert_eq!(summary.completed_at, execution.completed_at);
    assert_eq!(summary.task_count, 2);
    assert_eq!(summary.timestamp, execution.timestamp);
}

#[test]
fn test_workflow_execution_to_metadata() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowExecutionStatus::Running,
        ..Default::default()
    };

    let metadata = execution.to_metadata();

    assert_eq!(metadata.id, execution.id);
    assert_eq!(metadata.name, execution.workflow_name);
    assert_eq!(metadata.status, "running");
}

#[test]
fn test_workflow_execution_waiting_for_input_status() {

    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test Workflow",
            "tasks": []
        }),
        status: WorkflowExecutionStatus::WaitingForInput,
        created_at: Utc::now(),
        completed_at: None,
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    assert_eq!(execution.status, WorkflowExecutionStatus::WaitingForInput);
    assert_eq!(execution.status.as_str(), "waiting_for_input");


    let json = serde_json::to_string(&execution).unwrap();
    assert!(json.contains("waiting_for_input"));


    let summary = execution.to_summary();
    assert_eq!(summary.status, WorkflowExecutionStatus::WaitingForInput);
}

#[test]
fn test_workflow_execution_summary_default() {
    let summary = WorkflowExecutionSummary::default();

    assert!(!summary.id.is_empty());
    assert!(summary.workflow_name.is_empty());
    assert_eq!(summary.status, WorkflowExecutionStatus::Pending);
    assert_eq!(summary.task_count, 0);
}

#[test]
fn test_workflow_execution_summary_serialization() {
    let summary = WorkflowExecutionSummary {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowExecutionStatus::Failed,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        task_count: 5,
        timestamp: Utc::now(),
    };


    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("exec-1"));
    assert!(json.contains("failed"));
    assert!(json.contains("task_count"));


    let deserialized: WorkflowExecutionSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, summary.id);
    assert_eq!(deserialized.task_count, summary.task_count);
}

#[test]
fn test_workflow_metadata_default() {
    let metadata = WorkflowMetadata::default();

    assert!(!metadata.id.is_empty());
    assert!(metadata.name.is_empty());
    assert_eq!(metadata.status, "pending");
}

#[test]
fn test_workflow_metadata_serialization() {
    let metadata = WorkflowMetadata {
        id: "exec-1".to_string(),
        name: "Test Workflow".to_string(),
        status: "running".to_string(),
    };


    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("exec-1"));
    assert!(json.contains("running"));


    let deserialized: WorkflowMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, metadata.id);
    assert_eq!(deserialized.name, metadata.name);
    assert_eq!(deserialized.status, metadata.status);
}
