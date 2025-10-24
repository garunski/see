use s_e_e_core::engine::handlers::{
    cli_command::CliCommandHandler, cursor_agent::CursorAgentHandler,
};
use s_e_e_core::execution::context::ExecutionContext;
use s_e_e_core::task_executor::TaskExecutor;
use s_e_e_core::types::{TaskInfo, TaskStatus};
use serde_json::json;

#[tokio::test]
async fn test_cli_handler_with_persistence_helper() {
    // Create test context
    let tasks = vec![TaskInfo {
        id: "test_cli_task".to_string(),
        name: "Test CLI Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "test_execution".to_string(),
        "test_workflow".to_string(),
    );

    // Create handler - should not panic
    let _handler = CliCommandHandler::new(context.clone());

    // Verify handler was created successfully (no panic means success)
}

#[tokio::test]
async fn test_cursor_handler_with_persistence_helper() {
    // Create test context
    let tasks = vec![TaskInfo {
        id: "test_cursor_task".to_string(),
        name: "Test Cursor Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "test_execution".to_string(),
        "test_workflow".to_string(),
    );

    // Create handler - should not panic
    let _handler = CursorAgentHandler::new(context.clone());

    // Verify handler was created successfully (no panic means success)
}

#[tokio::test]
async fn test_cli_handler_execution_flow() {
    // Test that CLI handler can execute a simple command
    let tasks = vec![TaskInfo {
        id: "echo_task".to_string(),
        name: "Echo Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "test_execution".to_string(),
        "test_workflow".to_string(),
    );

    let handler = CliCommandHandler::new(context.clone());
    let logger = s_e_e_core::task_executor::ContextTaskLogger::new(context.clone());

    let task_config = json!({
        "task_id": "echo_task",
        "command": "echo",
        "args": ["hello"],
        "response_type": "text"
    });

    // Execute should complete without panic
    let result = TaskExecutor::execute(&handler, &task_config, &logger).await;

    // Verify result is Ok
    assert!(result.is_ok());
}
