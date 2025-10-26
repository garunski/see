//! Handler tests for the new workflow engine

use crate::handlers::{
    cli_command::CliCommandHandler, cursor_agent::CursorAgentHandler, custom::CustomHandler,
    user_input::UserInputHandler, HandlerRegistry, TaskHandler,
};
use crate::types::*;
use crate::HandlerError;
use serde_json::Value;

fn create_test_task(function: TaskFunction) -> EngineTask {
    EngineTask {
        id: "test_task".to_string(),
        name: "Test Task".to_string(),
        function,
        next_tasks: Vec::new(),
        status: TaskStatus::Pending,
    }
}

#[tokio::test]
async fn test_cli_command_handler() {
    let handler = CliCommandHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    let task = create_test_task(TaskFunction::CliCommand {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
    });

    let result = handler.execute(&mut context, &task).await.unwrap();

    assert!(result.success);
    assert!(result.output.as_str().unwrap().contains("hello"));
    assert!(context.per_task_logs.contains_key("test_task"));

    // Verify logs contain expected content
    let logs = &context.per_task_logs["test_task"];
    assert!(!logs.is_empty());
    assert!(logs.iter().any(|log| log.contains("Executing CLI command")));
    assert!(logs.iter().any(|log| log.contains("Output:")));
    assert!(logs.iter().any(|log| log.contains("hello")));
}

#[tokio::test]
async fn test_cli_command_handler_error() {
    let handler = CliCommandHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    let task = create_test_task(TaskFunction::CliCommand {
        command: "nonexistent_command_xyz".to_string(),
        args: vec![],
    });

    let result = handler.execute(&mut context, &task).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        HandlerError::ExecutionFailed(msg) => {
            assert!(msg.contains("No such file or directory"));
        }
        _ => panic!("Expected ExecutionFailed error"),
    }

    // Verify error logs were still created
    assert!(context.per_task_logs.contains_key("test_task"));

    // Verify error logs
    let logs = &context.per_task_logs["test_task"];
    assert!(!logs.is_empty());
    assert!(logs.iter().any(|log| log.contains("Executing CLI command")));
}

#[tokio::test]
async fn test_cursor_agent_invalid_function_type() {
    let handler = CursorAgentHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    // Create task with wrong function type (CliCommand instead of CursorAgent)
    let task = create_test_task(TaskFunction::CliCommand {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
    });

    let result = handler.execute(&mut context, &task).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        HandlerError::InvalidConfiguration(msg) => {
            assert!(msg.contains("Expected CursorAgent function"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_custom_handler() {
    let handler = CustomHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    let task = create_test_task(TaskFunction::Custom {
        name: "test_function".to_string(),
        input: Value::String("test input".to_string()),
    });

    let result = handler.execute(&mut context, &task).await.unwrap();

    assert!(result.success);
    assert!(result.output.as_str().unwrap().contains("test_function"));
}

#[test]
fn test_handler_registry() {
    let registry = HandlerRegistry::new();

    assert!(registry.get_handler("cli_command").is_some());
    assert!(registry.get_handler("cursor_agent").is_some());
    assert!(registry.get_handler("custom").is_some());
    assert!(registry.get_handler("user_input").is_some());
    assert!(registry.get_handler("unknown").is_none());
}

#[tokio::test]
async fn test_cli_handler_invalid_function() {
    let handler = CliCommandHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    // Create task with wrong function type (CursorAgent instead of CliCommand)
    let task = create_test_task(TaskFunction::CursorAgent {
        prompt: "test prompt".to_string(),
        config: Value::Object(serde_json::Map::new()),
    });

    let result = handler.execute(&mut context, &task).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        HandlerError::InvalidConfiguration(msg) => {
            assert!(msg.contains("Expected CliCommand function"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_custom_handler_invalid_function() {
    let handler = CustomHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    // Create task with wrong function type (CliCommand instead of Custom)
    let task = create_test_task(TaskFunction::CliCommand {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
    });

    let result = handler.execute(&mut context, &task).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        HandlerError::InvalidConfiguration(msg) => {
            assert!(msg.contains("Expected Custom function"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_user_input_handler() {
    let handler = UserInputHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    let task = create_test_task(TaskFunction::UserInput {
        prompt: "Please enter your name:".to_string(),
        input_type: "string".to_string(),
        required: true,
        default: None,
    });

    // Add task to context
    context.tasks.insert("test_task".to_string(), task.clone());

    let result = handler.execute(&mut context, &task).await.unwrap();

    // Should return success but waiting for input
    assert!(result.success);
    
    // Check output contains waiting_for_input flag
    assert!(result.output.get("waiting_for_input").unwrap().as_bool().unwrap());
    assert_eq!(
        result.output.get("prompt").unwrap().as_str().unwrap(),
        "Please enter your name:"
    );
    assert_eq!(
        result.output.get("input_type").unwrap().as_str().unwrap(),
        "string"
    );
    assert!(result.output.get("required").unwrap().as_bool().unwrap());
    
    // Check task status was updated
    assert_eq!(context.tasks.get("test_task").unwrap().status, TaskStatus::WaitingForInput);
    
    // Check logs were created
    assert!(context.per_task_logs.contains_key("test_task"));
    let logs = &context.per_task_logs["test_task"];
    assert!(logs.iter().any(|log| log.contains("Waiting for user input")));
}

#[tokio::test]
async fn test_user_input_handler_with_default() {
    let handler = UserInputHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    let task = create_test_task(TaskFunction::UserInput {
        prompt: "Enter a number:".to_string(),
        input_type: "number".to_string(),
        required: false,
        default: Some(Value::Number(serde_json::Number::from(42))),
    });

    let result = handler.execute(&mut context, &task).await.unwrap();

    assert!(result.success);
    assert!(result.output.get("waiting_for_input").unwrap().as_bool().unwrap());
    
    // Check default value is included
    let default_value = result.output.get("default");
    assert!(default_value.is_some());
}

#[tokio::test]
async fn test_user_input_handler_invalid_function() {
    let handler = UserInputHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());

    // Create task with wrong function type (CliCommand instead of UserInput)
    let task = create_test_task(TaskFunction::CliCommand {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
    });

    let result = handler.execute(&mut context, &task).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        HandlerError::InvalidConfiguration(msg) => {
            assert!(msg.contains("Expected UserInput function"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}
