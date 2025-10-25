//! Engine tests for the new workflow engine

use crate::engine::*;
use crate::parser::*;
use crate::types::*;

#[tokio::test]
async fn test_simple_workflow_execution() {
    let json = r#"
    {
        "id": "test",
        "name": "Test Workflow",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["hello"]
                    }
                }
            }
        ]
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();

    assert!(result.success);
    assert_eq!(result.workflow_name, "Test Workflow");
    assert_eq!(result.tasks.len(), 1);
    assert_eq!(result.tasks[0].id, "task1");
    assert_eq!(result.tasks[0].status, TaskStatus::Complete);
}

#[tokio::test]
async fn test_parallel_execution() {
    let json = r#"
    {
        "id": "parallel",
        "name": "Parallel Workflow",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["task1"]
                    }
                }
            },
            {
                "id": "task2",
                "name": "Task 2",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["task2"]
                    }
                }
            }
        ]
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();

    assert!(result.success);
    assert_eq!(result.tasks.len(), 2);
    assert!(result
        .tasks
        .iter()
        .all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_sequential_execution() {
    let json = r#"
    {
        "id": "sequential",
        "name": "Sequential Workflow",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["task1"]
                    }
                },
                "dependencies": []
            },
            {
                "id": "task2",
                "name": "Task 2",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["task2"]
                    }
                },
                "dependencies": ["task1"]
            }
        ]
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();

    assert!(result.success);
    assert_eq!(result.tasks.len(), 2);
    assert!(result
        .tasks
        .iter()
        .all(|t| t.status == TaskStatus::Complete));
}
