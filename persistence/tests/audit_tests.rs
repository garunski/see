//! Tests for audit store operations
//!
//! Tests log_audit_event following Single Responsibility Principle.

use chrono::Utc;
use persistence::{AuditEvent, AuditStatus, Store};

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

#[tokio::test]
async fn test_log_audit_event_success() {
    let store = create_test_store().await;

    let event = AuditEvent::success(
        "task-1".to_string(),
        "Task completed successfully".to_string(),
        5,
    );

    let result = store.log_audit_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_log_audit_event_failure() {
    let store = create_test_store().await;

    let event = AuditEvent::failure(
        "task-1".to_string(),
        "Task failed with error".to_string(),
        0,
    );

    let result = store.log_audit_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_log_audit_event_multiple() {
    let store = create_test_store().await;

    let events = vec![
        AuditEvent::success("task-1".to_string(), "Task 1 completed".to_string(), 3),
        AuditEvent::failure("task-2".to_string(), "Task 2 failed".to_string(), 0),
        AuditEvent::success("task-3".to_string(), "Task 3 completed".to_string(), 7),
    ];

    for event in events {
        let result = store.log_audit_event(event).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_audit_event_serialization() {
    let store = create_test_store().await;

    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "task-1".to_string(),
        status: AuditStatus::Success,
        timestamp: Utc::now(),
        changes_count: 5,
        message: "Task completed successfully".to_string(),
    };

    let result = store.log_audit_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_audit_event_validation_error() {
    // Create invalid event (empty task_id)
    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "".to_string(), // Invalid: empty task_id
        status: AuditStatus::Success,
        timestamp: Utc::now(),
        changes_count: 5,
        message: "Test message".to_string(),
    };

    // This should fail during validation
    let result = event.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task ID cannot be empty"));
}

#[tokio::test]
async fn test_audit_event_empty_message() {
    // Create invalid event (empty message)
    let event = AuditEvent {
        id: "audit-1".to_string(),
        task_id: "task-1".to_string(),
        status: AuditStatus::Success,
        timestamp: Utc::now(),
        changes_count: 5,
        message: "".to_string(), // Invalid: empty message
    };

    // This should fail during validation
    let result = event.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("message cannot be empty"));
}
