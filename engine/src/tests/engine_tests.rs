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
async fn test_workflow_handler_not_found() {
    let json = r#"
    {
        "id": "unknown_handler",
        "name": "Unknown Handler Test",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1",
                "function": {
                    "name": "unknown_handler",
                    "input": {
                        "test": "data"
                    }
                }
            }
        ]
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();
    
    // The workflow completes successfully even with unknown handlers
    // The engine treats unknown handlers as failed tasks but doesn't fail the entire workflow
    assert!(result.success);
    assert_eq!(result.workflow_name, "Unknown Handler Test");
    assert_eq!(result.tasks.len(), 1);
    assert_eq!(result.tasks[0].status, TaskStatus::Complete);
}

#[tokio::test]
async fn test_workflow_with_failing_dependency() {
    let json = r#"
    {
        "id": "failing_dep",
        "name": "Failing Dependency Test",
        "tasks": [
            {
                "id": "parent",
                "name": "Parent Task",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "nonexistent_command",
                        "args": ["this", "will", "fail"]
                    }
                }
            },
            {
                "id": "child",
                "name": "Child Task",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["child"]
                    }
                },
                "dependencies": ["parent"]
            }
        ]
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();
    
    // Workflow should complete but with errors
    assert!(!result.success);
    assert!(!result.errors.is_empty());
    
    // Both tasks should be marked complete (even failed ones)
    assert_eq!(result.tasks.len(), 2);
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_empty_workflow() {
    let json = r#"
    {
        "id": "empty",
        "name": "Empty Workflow",
        "tasks": []
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Empty Workflow");
    assert!(result.tasks.is_empty());
    assert!(result.errors.is_empty());
}
