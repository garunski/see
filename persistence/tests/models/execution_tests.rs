//! Tests for execution models
//! 
//! Tests WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata following Single Responsibility Principle.

use persistence::{WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata, WorkflowStatus, TaskExecution, TaskStatus};
use chrono::Utc;

#[test]
fn test_workflow_execution_default() {
    let execution = WorkflowExecution::default();
    
    assert!(!execution.id.is_empty());
    assert!(execution.workflow_name.is_empty());
    assert_eq!(execution.workflow_snapshot, serde_json::json!({}));
    assert_eq!(execution.status, WorkflowStatus::Pending);
    assert!(execution.created_at <= Utc::now());
    assert!(execution.completed_at.is_none());
    assert!(execution.success.is_none());
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
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };
    
    // Test serialization
    let json = serde_json::to_string(&execution).unwrap();
    assert!(json.contains("exec-1"));
    assert!(json.contains("Test Workflow"));
    assert!(json.contains("complete"));
    
    // Test deserialization
    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, execution.id);
    assert_eq!(deserialized.workflow_name, execution.workflow_name);
    assert_eq!(deserialized.status, execution.status);
    assert_eq!(deserialized.success, execution.success);
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
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
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
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
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
    assert_eq!(summary.success, execution.success);
    assert_eq!(summary.task_count, 2);
    assert_eq!(summary.timestamp, execution.timestamp);
}

#[test]
fn test_workflow_execution_to_metadata() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowStatus::Running,
        ..Default::default()
    };
    
    let metadata = execution.to_metadata();
    
    assert_eq!(metadata.id, execution.id);
    assert_eq!(metadata.name, execution.workflow_name);
    assert_eq!(metadata.status, "running");
}

#[test]
fn test_workflow_execution_summary_default() {
    let summary = WorkflowExecutionSummary::default();
    
    assert!(!summary.id.is_empty());
    assert!(summary.workflow_name.is_empty());
    assert_eq!(summary.status, WorkflowStatus::Pending);
    assert_eq!(summary.task_count, 0);
}

#[test]
fn test_workflow_execution_summary_serialization() {
    let summary = WorkflowExecutionSummary {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowStatus::Failed,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: false,
        task_count: 5,
        timestamp: Utc::now(),
    };
    
    // Test serialization
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("exec-1"));
    assert!(json.contains("failed"));
    assert!(json.contains("task_count"));
    
    // Test deserialization
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
    
    // Test serialization
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("exec-1"));
    assert!(json.contains("running"));
    
    // Test deserialization
    let deserialized: WorkflowMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, metadata.id);
    assert_eq!(deserialized.name, metadata.name);
    assert_eq!(deserialized.status, metadata.status);
}
