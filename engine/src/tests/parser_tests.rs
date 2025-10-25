//! Parser tests for the new workflow engine

use crate::parser::*;
use crate::errors::*;

#[test]
fn test_parse_simple_workflow() {
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
    assert_eq!(workflow.id, "test");
    assert_eq!(workflow.name, "Test Workflow");
    assert_eq!(workflow.tasks.len(), 1);
    assert_eq!(workflow.tasks[0].id, "task1");
}

#[test]
fn test_parse_nested_next_tasks() {
    let json = r#"
    {
        "id": "nested",
        "name": "Nested Workflow",
        "tasks": [
            {
                "id": "root",
                "name": "Root Task",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["root"]
                    }
                },
                "next_tasks": [
                    {
                        "id": "child1",
                        "name": "Child 1",
                        "function": {
                            "name": "cli_command",
                            "input": {
                                "command": "echo",
                                "args": ["child1"]
                            }
                        }
                    },
                    {
                        "id": "child2",
                        "name": "Child 2",
                        "function": {
                            "name": "cli_command",
                            "input": {
                                "command": "echo",
                                "args": ["child2"]
                            }
                        }
                    }
                ]
            }
        ]
    }
    "#;

    let workflow = parse_workflow(json).unwrap();
    assert_eq!(workflow.tasks.len(), 3); // root + 2 children
    
    // Find root task
    let root_task = workflow.tasks.iter().find(|t| t.id == "root").unwrap();
    assert_eq!(root_task.next_tasks.len(), 2);
    assert_eq!(root_task.dependencies.len(), 0);
    
    // Find child tasks
    let child1 = workflow.tasks.iter().find(|t| t.id == "child1").unwrap();
    let child2 = workflow.tasks.iter().find(|t| t.id == "child2").unwrap();
    
    assert_eq!(child1.dependencies, vec!["root"]);
    assert_eq!(child2.dependencies, vec!["root"]);
}

#[test]
fn test_parse_deeply_nested() {
    let json = r#"
    {
        "id": "deep",
        "name": "Deep Nested Workflow",
        "tasks": [
            {
                "id": "level1",
                "name": "Level 1",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["level1"]
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
                                "args": ["level2"]
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
                                        "args": ["level3"]
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

    let workflow = parse_workflow(json).unwrap();
    assert_eq!(workflow.tasks.len(), 3); // level1 + level2 + level3
    
    let level3 = workflow.tasks.iter().find(|t| t.id == "level3").unwrap();
    assert_eq!(level3.dependencies, vec!["level2"]);
}

#[test]
fn test_duplicate_task_ids() {
    let json = r#"
    {
        "id": "duplicate",
        "name": "Duplicate IDs",
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
            },
            {
                "id": "task1",
                "name": "Task 1 Duplicate",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["world"]
                    }
                }
            }
        ]
    }
    "#;

    let result = parse_workflow(json);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ParserError::InvalidTask(_)));
}
