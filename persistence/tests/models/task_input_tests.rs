//! Tests for TaskExecution with input fields
//!
//! Tests the new user input fields added to TaskExecution following Single Responsibility Principle.

use persistence::{TaskExecution, TaskStatus};
use chrono::Utc;

#[test]
fn test_task_execution_with_user_input_fields() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::WaitingForInput,
        output: None,
        error: None,
        created_at: Utc::now(),
        completed_at: None,
        user_input: Some("user provided value".to_string()),
        input_request_id: Some("request-123".to_string()),
        prompt_id: Some("prompt-456".to_string()),
    };
    
    assert_eq!(task.user_input, Some("user provided value".to_string()));
    assert_eq!(task.input_request_id, Some("request-123".to_string()));
    assert_eq!(task.prompt_id, Some("prompt-456".to_string()));
}

#[test]
fn test_task_execution_has_user_input() {
    let task_with_input = TaskExecution {
        user_input: Some("test".to_string()),
        ..Default::default()
    };
    assert!(task_with_input.has_user_input());
    
    let task_without_input = TaskExecution {
        user_input: None,
        ..Default::default()
    };
    assert!(!task_without_input.has_user_input());
}

#[test]
fn test_task_execution_get_input_request_id() {
    let task_with_request_id = TaskExecution {
        input_request_id: Some("request-123".to_string()),
        ..Default::default()
    };
    assert_eq!(task_with_request_id.get_input_request_id(), Some("request-123"));
    
    let task_without_request_id = TaskExecution {
        input_request_id: None,
        ..Default::default()
    };
    assert!(task_without_request_id.get_input_request_id().is_none());
}

#[test]
fn test_task_execution_with_input_serialization() {
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::WaitingForInput,
        output: None,
        error: None,
        created_at: Utc::now(),
        completed_at: None,
        user_input: Some("input value".to_string()),
        input_request_id: Some("request-id".to_string()),
        prompt_id: Some("prompt-id".to_string()),
    };
    
    // Test serialization
    let json = serde_json::to_string(&task).unwrap();
    assert!(json.contains("task-1"));
    assert!(json.contains("input value"));
    assert!(json.contains("request-id"));
    assert!(json.contains("prompt-id"));
    
    // Test deserialization
    let deserialized: TaskExecution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_input, task.user_input);
    assert_eq!(deserialized.input_request_id, task.input_request_id);
    assert_eq!(deserialized.prompt_id, task.prompt_id);
}

#[test]
fn test_task_execution_with_input_default() {
    let task = TaskExecution::default();
    
    assert!(task.user_input.is_none());
    assert!(task.input_request_id.is_none());
    assert!(task.prompt_id.is_none());
}

