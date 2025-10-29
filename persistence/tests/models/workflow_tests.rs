//! Tests for WorkflowDefinition model
//! 
//! Tests serialization, validation, defaults following Single Responsibility Principle.

use s_e_e_persistence::WorkflowDefinition;
use chrono::Utc;

#[test]
fn test_workflow_definition_default() {
    let workflow = WorkflowDefinition::default();
    
    assert!(!workflow.id.is_empty());
    assert!(workflow.name.is_empty());
    assert!(workflow.description.is_none());
    assert!(workflow.content.is_empty());
    assert!(!workflow.is_default);
    assert!(!workflow.is_edited);
    assert!(workflow.created_at <= Utc::now());
    assert!(workflow.updated_at <= Utc::now());
}

#[test]
fn test_workflow_definition_validation_success() {
    let workflow = WorkflowDefinition {
        id: "test-id".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("A test workflow".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let result = workflow.validate();
    assert!(result.is_ok());
}

#[test]
fn test_workflow_definition_validation_empty_id() {
    let workflow = WorkflowDefinition {
        id: "".to_string(),
        name: "Test Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    let result = workflow.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ID cannot be empty"));
}

#[test]
fn test_workflow_definition_validation_empty_name() {
    let workflow = WorkflowDefinition {
        id: "test-id".to_string(),
        name: "".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    let result = workflow.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name cannot be empty"));
}

#[test]
fn test_workflow_definition_validation_empty_content() {
    let workflow = WorkflowDefinition {
        id: "test-id".to_string(),
        name: "Test Workflow".to_string(),
        content: "".to_string(),
        ..Default::default()
    };
    
    let result = workflow.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("content cannot be empty"));
}

#[test]
fn test_workflow_definition_validation_invalid_json() {
    let workflow = WorkflowDefinition {
        id: "test-id".to_string(),
        name: "Test Workflow".to_string(),
        content: "invalid json".to_string(),
        ..Default::default()
    };
    
    let result = workflow.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid JSON content"));
}

#[test]
fn test_workflow_definition_serialization() {
    let workflow = WorkflowDefinition {
        id: "test-id".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("A test workflow".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        is_default: true,
        is_edited: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Test serialization
    let json = serde_json::to_string(&workflow).unwrap();
    assert!(json.contains("test-id"));
    assert!(json.contains("Test Workflow"));
    assert!(json.contains("A test workflow"));
    assert!(json.contains("is_default"));
    
    // Test deserialization
    let deserialized: WorkflowDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, workflow.id);
    assert_eq!(deserialized.name, workflow.name);
    assert_eq!(deserialized.description, workflow.description);
    assert_eq!(deserialized.content, workflow.content);
    assert_eq!(deserialized.is_default, workflow.is_default);
    assert_eq!(deserialized.is_edited, workflow.is_edited);
}

#[test]
fn test_workflow_definition_get_name() {
    let workflow = WorkflowDefinition {
        name: "My Workflow".to_string(),
        ..Default::default()
    };
    
    assert_eq!(workflow.get_name(), "My Workflow");
}

#[test]
fn test_workflow_definition_get_default_workflows() {
    let defaults = WorkflowDefinition::get_default_workflows();
    
    assert_eq!(defaults.len(), 3);
    assert!(defaults.iter().all(|w| w.is_default));
    
    let ids: Vec<&str> = defaults.iter().map(|w| w.id.as_str()).collect();
    assert!(ids.contains(&"default-simple"));
    assert!(ids.contains(&"default-parallel"));
    assert!(ids.contains(&"default-nested"));
}
