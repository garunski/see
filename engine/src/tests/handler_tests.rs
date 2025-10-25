//! Handler tests for the new workflow engine

use crate::handlers::{HandlerRegistry, get_function_type, TaskHandler, cli_command::CliCommandHandler, cursor_agent::CursorAgentHandler, custom::CustomHandler};
use crate::types::*;
use serde_json::Value;

fn create_test_task(function: TaskFunction) -> EngineTask {
    EngineTask {
        id: "test_task".to_string(),
        name: "Test Task".to_string(),
        function,
        next_tasks: Vec::new(),
        dependencies: Vec::new(),
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
}

#[tokio::test]
async fn test_cursor_agent_handler() {
    let handler = CursorAgentHandler;
    let mut context = ExecutionContext::new("test".to_string(), "test_workflow".to_string());
    
    let task = create_test_task(TaskFunction::CursorAgent {
        prompt: "test prompt".to_string(),
        config: Value::Object(serde_json::Map::new()),
    });

    let result = handler.execute(&mut context, &task).await.unwrap();
    
    assert!(result.success);
    assert!(result.output.as_str().unwrap().contains("test prompt"));
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
    assert!(registry.get_handler("unknown").is_none());
}

#[test]
fn test_get_function_type() {
    let cli_task = create_test_task(TaskFunction::CliCommand {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
    });
    
    let agent_task = create_test_task(TaskFunction::CursorAgent {
        prompt: "test".to_string(),
        config: Value::Null,
    });

    assert_eq!(get_function_type(&cli_task), "cli_command");
    assert_eq!(get_function_type(&agent_task), "cursor_agent");
}
