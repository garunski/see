// Workflow conversion tests ONLY

use core::bridge::*;
use s_e_e_persistence::WorkflowDefinition;

#[test]
fn test_workflow_result_creation() {
    let result = WorkflowResult {
        success: true,
        workflow_name: "Test Workflow".to_string(),
        execution_id: "exec-123".to_string(),
        tasks: vec![],
        audit_trail: vec![],
        per_task_logs: std::collections::HashMap::new(),
        errors: vec![],
    };
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Test Workflow");
    assert_eq!(result.execution_id, "exec-123");
}

#[test]
fn test_output_callback_type() {
    let callback: OutputCallback = std::sync::Arc::new(|msg: String| {
        assert_eq!(msg, "test");
    });
    
    callback("test".to_string());
}

#[test]
fn test_workflow_definition_to_engine_valid() {
    let workflow = WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("Test description".to_string()),
        content: r#"{
            "id": "test-workflow",
            "name": "Test Workflow",
            "tasks": [
                {
                    "id": "task-1",
                    "name": "Echo Hello",
                    "function": {
                        "cli_command": {
                            "command": "echo",
                            "args": ["Hello, World!"]
                        }
                    },
                    "next_tasks": []
                }
            ]
        }"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let result = core::bridge::workflow::workflow_definition_to_engine(&workflow);
    assert!(result.is_ok(), "Valid workflow should convert successfully");
    
    let engine_workflow = result.unwrap();
    assert_eq!(engine_workflow.id, "test-workflow");
    assert_eq!(engine_workflow.name, "Test Workflow");
    assert_eq!(engine_workflow.tasks.len(), 1);
    assert_eq!(engine_workflow.tasks[0].id, "task-1");
}

#[test]
fn test_workflow_definition_to_engine_invalid_json() {
    let workflow = WorkflowDefinition {
        id: "invalid-workflow".to_string(),
        name: "Invalid Workflow".to_string(),
        description: None,
        content: "invalid json".to_string(),
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let result = core::bridge::workflow::workflow_definition_to_engine(&workflow);
    assert!(result.is_err(), "Invalid JSON should fail conversion");
    
    match result.unwrap_err() {
        core::CoreError::Engine(s_e_e_engine::EngineError::Parser(_)) => {}, // Expected
        other => panic!("Expected Parser error, got: {:?}", other),
    }
}

#[test]
fn test_workflow_definition_to_engine_missing_fields() {
    let workflow = WorkflowDefinition {
        id: "incomplete-workflow".to_string(),
        name: "Incomplete Workflow".to_string(),
        description: None,
        content: r#"{"name": "Test"}"#.to_string(), // Missing id and tasks
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let result = core::bridge::workflow::workflow_definition_to_engine(&workflow);
    assert!(result.is_err(), "Missing fields should fail conversion");
    
    match result.unwrap_err() {
        core::CoreError::Engine(s_e_e_engine::EngineError::Parser(_)) => {}, // Expected
        other => panic!("Expected Parser error, got: {:?}", other),
    }
}
