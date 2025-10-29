//! Tests for all enums
//! 
//! Tests serialization, variants following Single Responsibility Principle.

use s_e_e_persistence::{WorkflowStatus, Theme, TaskStatus, AuditStatus};

#[test]
fn test_workflow_status_variants() {
    assert_eq!(WorkflowExecutionStatus::Pending.to_string(), "pending");
    assert_eq!(WorkflowExecutionStatus::Running.to_string(), "running");
    assert_eq!(WorkflowExecutionStatus::Complete.to_string(), "complete");
    assert_eq!(WorkflowExecutionStatus::Failed.to_string(), "failed");
}

#[test]
fn test_workflow_status_serialization() {
    let status = WorkflowExecutionStatus::Running;
    
    // Test serialization
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"running\"");
    
    // Test deserialization
    let deserialized: WorkflowStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, WorkflowExecutionStatus::Running);
}

#[test]
fn test_workflow_status_all_variants_serialization() {
    let variants = vec![
        WorkflowExecutionStatus::Pending,
        WorkflowExecutionStatus::Running,
        WorkflowExecutionStatus::Complete,
        WorkflowExecutionStatus::Failed,
    ];
    
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: WorkflowStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn test_theme_variants() {
    // Test that themes serialize to expected strings
    let light_json = serde_json::to_string(&Theme::Light).unwrap();
    assert_eq!(light_json, "\"light\"");
    
    let dark_json = serde_json::to_string(&Theme::Dark).unwrap();
    assert_eq!(dark_json, "\"dark\"");
    
    let system_json = serde_json::to_string(&Theme::System).unwrap();
    assert_eq!(system_json, "\"system\"");
}

#[test]
fn test_theme_serialization() {
    let theme = Theme::Dark;
    
    // Test serialization
    let json = serde_json::to_string(&theme).unwrap();
    assert_eq!(json, "\"dark\"");
    
    // Test deserialization
    let deserialized: Theme = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, Theme::Dark);
}

#[test]
fn test_theme_all_variants_serialization() {
    let variants = vec![Theme::Light, Theme::Dark, Theme::System];
    
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn test_task_status_variants() {
    // Test that task statuses serialize to expected strings
    let pending_json = serde_json::to_string(&TaskExecutionStatus::Pending).unwrap();
    assert_eq!(pending_json, "\"pending\"");
    
    let in_progress_json = serde_json::to_string(&TaskExecutionStatus::InProgress).unwrap();
    assert_eq!(in_progress_json, "\"in_progress\"");
    
    let complete_json = serde_json::to_string(&TaskExecutionStatus::Complete).unwrap();
    assert_eq!(complete_json, "\"complete\"");
    
    let failed_json = serde_json::to_string(&TaskExecutionStatus::Failed).unwrap();
    assert_eq!(failed_json, "\"failed\"");
    
    let waiting_json = serde_json::to_string(&TaskExecutionStatus::WaitingForInput).unwrap();
    assert_eq!(waiting_json, "\"waiting_for_input\"");
}

#[test]
fn test_task_status_serialization() {
    let status = TaskExecutionStatus::InProgress;
    
    // Test serialization
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"in_progress\"");
    
    // Test deserialization
    let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, TaskExecutionStatus::InProgress);
}

#[test]
fn test_task_status_all_variants_serialization() {
    let variants = vec![
        TaskExecutionStatus::Pending,
        TaskExecutionStatus::InProgress,
        TaskExecutionStatus::Complete,
        TaskExecutionStatus::Failed,
        TaskExecutionStatus::WaitingForInput,
    ];
    
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn test_audit_status_variants() {
    // Test that audit statuses serialize to expected strings
    let success_json = serde_json::to_string(&AuditStatus::Success).unwrap();
    assert_eq!(success_json, "\"success\"");
    
    let failure_json = serde_json::to_string(&AuditStatus::Failure).unwrap();
    assert_eq!(failure_json, "\"failure\"");
}

#[test]
fn test_audit_status_serialization() {
    let status = AuditStatus::Success;
    
    // Test serialization
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"success\"");
    
    // Test deserialization
    let deserialized: AuditStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, AuditStatus::Success);
}

#[test]
fn test_audit_status_all_variants_serialization() {
    let variants = vec![AuditStatus::Success, AuditStatus::Failure];
    
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: AuditStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }
}
