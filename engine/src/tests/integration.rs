//! Integration tests for the new workflow engine

use crate::*;
use std::fs;

#[tokio::test]
async fn test_simple_workflow_from_file() {
    let json = fs::read_to_string("examples/simple.json").unwrap();
    let result = execute_workflow_from_json(&json).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Simple Sequential Workflow");
    assert_eq!(result.tasks.len(), 2);
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_parallel_workflow_from_file() {
    let json = fs::read_to_string("examples/parallel.json").unwrap();
    let result = execute_workflow_from_json(&json).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Parallel Execution Workflow");
    assert_eq!(result.tasks.len(), 4); // root + 3 parallel tasks
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_nested_workflow_from_file() {
    let json = fs::read_to_string("examples/nested.json").unwrap();
    let result = execute_workflow_from_json(&json).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Nested Dependencies Workflow");
    assert_eq!(result.tasks.len(), 6); // root + level1a + level1b + level2a + level2b + level2c
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_mixed_function_types() {
    let json = r#"
    {
        "id": "mixed",
        "name": "Mixed Function Types",
        "tasks": [
            {
                "id": "cli_task",
                "name": "CLI Task",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["CLI task executed"]
                    }
                },
                "next_tasks": [
                    {
                        "id": "agent_task",
                        "name": "Agent Task",
                        "function": {
                            "name": "cursor_agent",
                            "input": {
                                "prompt": "Test prompt",
                                "config": {}
                            }
                        },
                        "next_tasks": [
                            {
                                "id": "custom_task",
                                "name": "Custom Task",
                                "function": {
                                    "name": "custom",
                                    "input": {
                                        "name": "test_function",
                                        "input": {"test": "data"}
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        ]
    }
    "#;

    let result = execute_workflow_from_json(json).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Mixed Function Types");
    assert_eq!(result.tasks.len(), 3);
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_error_handling() {
    let json = r#"
    {
        "id": "error_test",
        "name": "Error Handling Test",
        "tasks": [
            {
                "id": "failing_task",
                "name": "Failing Task",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "nonexistent_command",
                        "args": ["this", "will", "fail"]
                    }
                }
            }
        ]
    }
    "#;

    let result = execute_workflow_from_json(json).await.unwrap();
    
    // The workflow should complete but with errors
    assert!(!result.success);
    assert!(!result.errors.is_empty());
    assert_eq!(result.tasks[0].status, TaskStatus::Complete); // Task is marked complete even if it failed
}

#[tokio::test]
async fn test_large_parallel_workflow() {
    let mut tasks = Vec::new();
    
    // Create 10 parallel tasks
    for i in 1..=10 {
        tasks.push(format!(
            r#"{{
                "id": "task_{}",
                "name": "Task {}",
                "function": {{
                    "name": "cli_command",
                    "input": {{
                        "command": "echo",
                        "args": ["Task {} executing"]
                    }}
                }}
            }}"#,
            i, i, i
        ));
    }
    
    let json = format!(
        r#"{{
            "id": "large_parallel",
            "name": "Large Parallel Workflow",
            "tasks": [
                {{
                    "id": "root",
                    "name": "Root Task",
                    "function": {{
                        "name": "cli_command",
                        "input": {{
                            "command": "echo",
                            "args": ["Starting large parallel workflow"]
                        }}
                    }},
                    "next_tasks": [{}]
                }}
            ]
        }}"#,
        tasks.join(",")
    );

    let result = execute_workflow_from_json(&json).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Large Parallel Workflow");
    assert_eq!(result.tasks.len(), 11); // root + 10 parallel tasks
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}

#[tokio::test]
async fn test_deep_nesting() {
    let json = r#"
    {
        "id": "deep_nesting",
        "name": "Deep Nesting Test",
        "tasks": [
            {
                "id": "level1",
                "name": "Level 1",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["Level 1"]
                    }
                },
                "next_tasks": [
                    {
                        "id": "level2",
                        "name": "Level 2",
                        "function": {
                            "name": "cli_command",
                            "input": {
                                "command": "echo",
                                "args": ["Level 2"]
                            }
                        },
                        "next_tasks": [
                            {
                                "id": "level3",
                                "name": "Level 3",
                                "function": {
                                    "name": "cli_command",
                                    "input": {
                                        "command": "echo",
                                        "args": ["Level 3"]
                                    }
                                },
                                "next_tasks": [
                                    {
                                        "id": "level4",
                                        "name": "Level 4",
                                        "function": {
                                            "name": "cli_command",
                                            "input": {
                                                "command": "echo",
                                                "args": ["Level 4"]
                                            }
                                        },
                                        "next_tasks": [
                                            {
                                                "id": "level5",
                                                "name": "Level 5",
                                                "function": {
                                                    "name": "cli_command",
                                                    "input": {
                                                        "command": "echo",
                                                        "args": ["Level 5"]
                                                    }
                                                }
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        ]
    }
    "#;

    let result = execute_workflow_from_json(json).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.workflow_name, "Deep Nesting Test");
    assert_eq!(result.tasks.len(), 5); // level1 through level5
    assert!(result.tasks.iter().all(|t| t.status == TaskStatus::Complete));
}
