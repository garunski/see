// Error handling tests ONLY

use s_e_e_engine::{EngineError, HandlerError, ParserError};
use s_e_e_core::CoreError;

#[test]
fn test_core_error_engine_conversion() {
    // Test Engine error conversion
    let parser_error = ParserError::MissingField("id".to_string());
    let engine_error = EngineError::Parser(parser_error);
    let core_error: CoreError = engine_error.into();

    match core_error {
        CoreError::Engine(EngineError::Parser(ParserError::MissingField(field))) => {
            assert_eq!(field, "id");
        }
        other => panic!("Expected Engine Parser error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_handler_conversion() {
    let handler_error = HandlerError::ExecutionFailed("command failed".to_string());
    let engine_error = EngineError::Handler(handler_error);
    let core_error: CoreError = engine_error.into();

    match core_error {
        CoreError::Engine(EngineError::Handler(HandlerError::ExecutionFailed(msg))) => {
            assert_eq!(msg, "command failed");
        }
        other => panic!("Expected Engine Handler error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_execution_conversion() {
    let engine_error = EngineError::Execution("workflow failed".to_string());
    let core_error: CoreError = engine_error.into();

    match core_error {
        CoreError::Engine(EngineError::Execution(msg)) => {
            assert_eq!(msg, "workflow failed");
        }
        other => panic!("Expected Engine Execution error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_string_conversion() {
    // Test String to Persistence error conversion
    let persistence_error: CoreError = "test persistence error".to_string().into();

    match persistence_error {
        CoreError::Persistence(msg) => {
            assert_eq!(msg, "test persistence error");
        }
        other => panic!("Expected Persistence error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_workflow_not_found() {
    let error = CoreError::WorkflowNotFound("workflow-123".to_string());

    match error {
        CoreError::WorkflowNotFound(id) => {
            assert_eq!(id, "workflow-123");
        }
        other => panic!("Expected WorkflowNotFound error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_task_not_found() {
    let error = CoreError::TaskNotFound("task-456".to_string());

    match error {
        CoreError::TaskNotFound(id) => {
            assert_eq!(id, "task-456");
        }
        other => panic!("Expected TaskNotFound error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_execution() {
    let error = CoreError::Execution("execution failed".to_string());

    match error {
        CoreError::Execution(msg) => {
            assert_eq!(msg, "execution failed");
        }
        other => panic!("Expected Execution error, got: {:?}", other),
    }
}

#[test]
fn test_core_error_display() {
    // Test that all error variants implement Display correctly
    let errors = vec![
        CoreError::Engine(EngineError::Parser(ParserError::MissingField(
            "test".to_string(),
        ))),
        CoreError::Persistence("test persistence error".to_string()),
        CoreError::WorkflowNotFound("workflow-123".to_string()),
        CoreError::TaskNotFound("task-456".to_string()),
        CoreError::Execution("execution failed".to_string()),
    ];

    for error in errors {
        let error_msg = format!("{}", error);
        assert!(!error_msg.is_empty(), "Error message should not be empty");
        assert!(error_msg.len() > 10, "Error message should be descriptive");
    }
}

#[test]
fn test_core_error_debug() {
    // Test that all error variants implement Debug correctly
    let error = CoreError::WorkflowNotFound("test-workflow".to_string());
    let debug_msg = format!("{:?}", error);

    assert!(debug_msg.contains("WorkflowNotFound"));
    assert!(debug_msg.contains("test-workflow"));
}
