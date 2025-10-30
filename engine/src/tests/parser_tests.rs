use crate::errors::*;
use crate::parser::*;

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
    assert_eq!(workflow.tasks.len(), 3);

    let root_task = workflow.tasks.iter().find(|t| t.id == "root").unwrap();
    assert_eq!(root_task.next_tasks.len(), 2);

    let _child1 = workflow.tasks.iter().find(|t| t.id == "child1").unwrap();
    let _child2 = workflow.tasks.iter().find(|t| t.id == "child2").unwrap();

    assert!(root_task.next_tasks.iter().any(|t| t.id == "child1"));
    assert!(root_task.next_tasks.iter().any(|t| t.id == "child2"));
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
    assert_eq!(workflow.tasks.len(), 3);

    let _level3 = workflow.tasks.iter().find(|t| t.id == "level3").unwrap();

    let level2 = workflow.tasks.iter().find(|t| t.id == "level2").unwrap();
    assert!(level2.next_tasks.iter().any(|t| t.id == "level3"));
}

#[test]
fn test_parse_invalid_json() {
    let json = r#"{ invalid json }"#;
    let result = parse_workflow(json);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ParserError::Json(_)));
}

#[test]
fn test_parse_missing_tasks_field() {
    let json = r#"
    {
        "id": "test",
        "name": "Test Workflow"
    }
    "#;
    let result = parse_workflow(json);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParserError::MissingField(field) => assert_eq!(field, "tasks"),
        _ => panic!("Expected MissingField error for tasks"),
    }
}

#[test]
fn test_parse_missing_function_field() {
    let json = r#"
    {
        "id": "test",
        "name": "Test Workflow",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1"
            }
        ]
    }
    "#;
    let result = parse_workflow(json);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParserError::MissingField(field) => assert_eq!(field, "function"),
        _ => panic!("Expected MissingField error for function"),
    }
}

#[test]
fn test_parse_missing_command() {
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
                        "args": ["hello"]
                    }
                }
            }
        ]
    }
    "#;
    let result = parse_workflow(json);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParserError::MissingField(field) => assert_eq!(field, "function.input.command"),
        _ => panic!("Expected MissingField error for command"),
    }
}

#[test]
fn test_parse_missing_prompt() {
    let json = r#"
    {
        "id": "test",
        "name": "Test Workflow",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1",
                "function": {
                    "name": "cursor_agent",
                    "input": {
                        "config": {}
                    }
                }
            }
        ]
    }
    "#;
    let result = parse_workflow(json);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParserError::MissingField(field) => assert_eq!(field, "function.input.prompt"),
        _ => panic!("Expected MissingField error for prompt"),
    }
}

#[test]
fn test_parse_circular_dependency_parent_child() {
    let json = r#"
    {
        "id": "circular",
        "name": "Circular Dependency",
        "tasks": [
            {
                "id": "parent",
                "name": "Parent Task",
                "function": {
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["parent"]
                    }
                },
                "next_tasks": [
                    {
                        "id": "child",
                        "name": "Child Task",
                        "function": {
                            "name": "cli_command",
                            "input": {
                                "command": "echo",
                                "args": ["child"]
                            }
                        }
                    }
                ]
            }
        ]
    }
    "#;
    let workflow = parse_workflow(json).unwrap();

    assert_eq!(workflow.tasks.len(), 2);
}
