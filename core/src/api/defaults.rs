// Default workflows ONLY

use persistence::WorkflowDefinition;
use chrono::Utc;

/// Get default workflow templates
pub fn get_default_workflows() -> Vec<WorkflowDefinition> {
    let now = Utc::now();
    
    vec![
        WorkflowDefinition {
            id: "default-simple".to_string(),
            name: "Simple Workflow".to_string(),
            description: Some("A simple workflow with one task".to_string()),
            content: r#"{
                "id": "simple-workflow",
                "name": "Simple Workflow",
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
            is_default: true,
            is_edited: false,
            created_at: now,
            updated_at: now,
        },
        WorkflowDefinition {
            id: "default-parallel".to_string(),
            name: "Parallel Workflow".to_string(),
            description: Some("A workflow with parallel tasks".to_string()),
            content: r#"{
                "id": "parallel-workflow",
                "name": "Parallel Workflow",
                "tasks": [
                    {
                        "id": "task-1",
                        "name": "Task 1",
                        "function": {
                            "cli_command": {
                                "command": "echo",
                                "args": ["Task 1 complete"]
                            }
                        },
                        "next_tasks": []
                    },
                    {
                        "id": "task-2",
                        "name": "Task 2",
                        "function": {
                            "cli_command": {
                                "command": "echo",
                                "args": ["Task 2 complete"]
                            }
                        },
                        "next_tasks": []
                    }
                ]
            }"#.to_string(),
            is_default: true,
            is_edited: false,
            created_at: now,
            updated_at: now,
        },
        WorkflowDefinition {
            id: "default-nested".to_string(),
            name: "Nested Workflow".to_string(),
            description: Some("A workflow with nested task dependencies".to_string()),
            content: r#"{
                "id": "nested-workflow",
                "name": "Nested Workflow",
                "tasks": [
                    {
                        "id": "task-1",
                        "name": "First Task",
                        "function": {
                            "cli_command": {
                                "command": "echo",
                                "args": ["First task complete"]
                            }
                        },
                        "next_tasks": [
                            {
                                "id": "task-2",
                                "name": "Second Task",
                                "function": {
                                    "cli_command": {
                                        "command": "echo",
                                        "args": ["Second task complete"]
                                    }
                                },
                                "next_tasks": []
                            }
                        ]
                    }
                ]
            }"#.to_string(),
            is_default: true,
            is_edited: false,
            created_at: now,
            updated_at: now,
        },
    ]
}
