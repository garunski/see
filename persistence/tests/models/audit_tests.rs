//! Tests for AuditEvent model
//! 
//! Tests serialization, validation following Single Responsibility Principle.

use s_e_e_persistence::{AuditEvent, AuditStatus};
use chrono::Utc;

#[test]
fn test_audit_event_default() {
    let event = AuditEvent::default();
    
    assert!(!event.id.is_empty());
    assert!(event.task_id.is_empty());
    assert_eq!(event.status, AuditStatus::Success);
    assert!(event.timestamp <= Utc::now());
    assert_eq!(event.changes_count, 0);
    assert!(event.message.is_empty());
}

#[test]
fn test_audit_event_validation_success() {
    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "task-1".to_string(),
        status: AuditStatus::Success,
        timestamp: Utc::now(),
        changes_count: 5,
        message: "Task completed successfully".to_string(),
    };
    
    let result = event.validate();
    assert!(result.is_ok());
}

#[test]
fn test_audit_event_validation_empty_id() {
    let event = AuditEvent {
        id: "".to_string(),
        task_id: "task-1".to_string(),
        message: "Test message".to_string(),
        ..Default::default()
    };
    
    let result = event.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ID cannot be empty"));
}

#[test]
fn test_audit_event_validation_empty_task_id() {
    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "".to_string(),
        message: "Test message".to_string(),
        ..Default::default()
    };
    
    let result = event.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task ID cannot be empty"));
}

#[test]
fn test_audit_event_validation_empty_message() {
    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "task-1".to_string(),
        message: "".to_string(),
        ..Default::default()
    };
    
    let result = event.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("message cannot be empty"));
}

#[test]
fn test_audit_event_success_constructor() {
    let event = AuditEvent::success(
        "task-1".to_string(),
        "Task completed successfully".to_string(),
        3
    );
    
    assert!(!event.id.is_empty());
    assert_eq!(event.task_id, "task-1");
    assert_eq!(event.status, AuditStatus::Success);
    assert_eq!(event.changes_count, 3);
    assert_eq!(event.message, "Task completed successfully");
    assert!(event.timestamp <= Utc::now());
}

#[test]
fn test_audit_event_failure_constructor() {
    let event = AuditEvent::failure(
        "task-1".to_string(),
        "Task failed with error".to_string(),
        0
    );
    
    assert!(!event.id.is_empty());
    assert_eq!(event.task_id, "task-1");
    assert_eq!(event.status, AuditStatus::Failure);
    assert_eq!(event.changes_count, 0);
    assert_eq!(event.message, "Task failed with error");
    assert!(event.timestamp <= Utc::now());
}

#[test]
fn test_audit_event_serialization() {
    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "task-1".to_string(),
        status: AuditStatus::Success,
        timestamp: Utc::now(),
        changes_count: 5,
        message: "Task completed successfully".to_string(),
    };
    
    // Test serialization
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("audit-1"));
    assert!(json.contains("task-1"));
    assert!(json.contains("success"));
    assert!(json.contains("Task completed successfully"));
    
    // Test deserialization
    let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, event.id);
    assert_eq!(deserialized.task_id, event.task_id);
    assert_eq!(deserialized.status, event.status);
    assert_eq!(deserialized.changes_count, event.changes_count);
    assert_eq!(deserialized.message, event.message);
}
