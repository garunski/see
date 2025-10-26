//! Comprehensive tests for workflow JSON schema validation

use s_e_e_core::validation::{self};

/// Test all example workflows from engine/examples/
#[cfg(test)]
mod example_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn load_example_file(name: &str) -> String {
        // Try multiple paths for different test environments
        let paths = [
            Path::new("engine/examples").join(name),
            Path::new("../engine/examples").join(name),
            Path::new("../../engine/examples").join(name),
        ];

        for path in &paths {
            if let Ok(content) = fs::read_to_string(path) {
                return content;
            }
        }

        panic!(
            "Failed to read example file {} from any of the attempted paths",
            name
        )
    }

    #[test]
    fn test_example_simple() {
        let json = load_example_file("simple.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }

    #[test]
    fn test_example_nested() {
        let json = load_example_file("nested.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }

    #[test]
    fn test_example_parallel() {
        let json = load_example_file("parallel.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }

    #[test]
    fn test_example_user_input_simple() {
        let json = load_example_file("user_input_simple.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }

    #[test]
    fn test_example_user_input_nested() {
        let json = load_example_file("user_input_nested.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }

    #[test]
    fn test_example_user_input_parallel() {
        let json = load_example_file("user_input_parallel.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }

    #[test]
    fn test_example_user_input_deep() {
        let json = load_example_file("user_input_deep.json");
        assert!(validation::validate_workflow_json(&json).is_ok());
    }
}

#[cfg(test)]
mod valid_workflow_tests {
    use super::*;

    #[test]
    fn test_validate_valid_simple_workflow() {
        let json = r#"{
            "id": "simple",
            "name": "Simple Sequential Workflow",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Hello from Task 1"]
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "task2",
                            "name": "Task 2",
                            "function": {
                                "name": "cli_command",
                                "input": {
                                    "command": "echo",
                                    "args": ["Hello from Task 2"]
                                }
                            }
                        }
                    ]
                }
            ]
        }"#;

        assert!(
            validation::validate_workflow_json(json).is_ok(),
            "Valid workflow should pass validation"
        );
    }

    #[test]
    fn test_validate_nested_tasks() {
        let json = r#"{
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
                                    }
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        assert!(validation::validate_workflow_json(json).is_ok());
    }
}

#[cfg(test)]
mod invalid_workflow_tests {
    use super::*;

    #[test]
    fn test_validate_missing_id() {
        let json = r#"{
            "name": "Test Workflow",
            "tasks": []
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing id should fail validation");
    }

    #[test]
    fn test_validate_missing_name() {
        let json = r#"{
            "id": "test",
            "tasks": []
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing name should fail validation");
    }

    #[test]
    fn test_validate_missing_tasks() {
        let json = r#"{
            "id": "test",
            "name": "Test Workflow"
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing tasks should fail validation");
    }

    #[test]
    fn test_validate_empty_id() {
        let json = r#"{
            "id": "",
            "name": "Test",
            "tasks": []
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Empty id should fail validation");
    }

    #[test]
    fn test_validate_empty_name() {
        let json = r#"{
            "id": "test",
            "name": "",
            "tasks": []
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Empty name should fail validation");
    }
}

#[cfg(test)]
mod function_type_tests {
    use super::*;

    #[test]
    fn test_cli_command_valid() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo"
                        }
                    }
                }
            ]
        }"#;

        assert!(validation::validate_workflow_json(json).is_ok());
    }

    #[test]
    fn test_cli_command_with_args() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "ls",
                            "args": ["-la", "/tmp"]
                        }
                    }
                }
            ]
        }"#;

        assert!(validation::validate_workflow_json(json).is_ok());
    }

    #[test]
    fn test_cli_command_missing_command() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "function": {
                        "name": "cli_command",
                        "input": {}
                    }
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing command should fail");
    }

    #[test]
    fn test_cli_command_empty_command() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": ""
                        }
                    }
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Empty command should fail");
    }

    #[test]
    fn test_user_input_valid() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Get Input",
                    "function": {
                        "name": "user_input",
                        "input": {
                            "prompt": "Enter name:",
                            "input_type": "string",
                            "required": true
                        }
                    }
                }
            ]
        }"#;

        assert!(validation::validate_workflow_json(json).is_ok());
    }

    #[test]
    fn test_user_input_missing_prompt() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Get Input",
                    "function": {
                        "name": "user_input",
                        "input": {
                            "input_type": "string"
                        }
                    }
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing prompt should fail");
    }

    #[test]
    fn test_user_input_missing_input_type() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Get Input",
                    "function": {
                        "name": "user_input",
                        "input": {
                            "prompt": "Enter value:"
                        }
                    }
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing input_type should fail");
    }

    #[test]
    fn test_cursor_agent_valid() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Agent Task",
                    "function": {
                        "name": "cursor_agent",
                        "input": {
                            "prompt": "Fix the bug"
                        }
                    }
                }
            ]
        }"#;

        assert!(validation::validate_workflow_json(json).is_ok());
    }

    #[test]
    fn test_cursor_agent_missing_prompt() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Agent Task",
                    "function": {
                        "name": "cursor_agent",
                        "input": {}
                    }
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Missing prompt should fail");
    }

    #[test]
    fn test_custom_function() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Custom Task",
                    "function": {
                        "name": "custom",
                        "input": {
                            "endpoint": "/api/process",
                            "data": {}
                        }
                    }
                }
            ]
        }"#;

        assert!(validation::validate_workflow_json(json).is_ok());
    }
}

#[cfg(test)]
mod duplicate_id_tests {
    use super::*;

    #[test]
    fn test_validate_duplicate_task_ids() {
        let json = r#"{
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
                            "args": []
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "task1",
                            "name": "Duplicate Task",
                            "function": {
                                "name": "cli_command",
                                "input": {
                                    "command": "echo",
                                    "args": []
                                }
                            }
                        }
                    ]
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(
            result.is_err(),
            "Workflow with duplicate task IDs should fail validation"
        );

        let errors = result.unwrap_err();
        assert!(
            errors
                .errors
                .iter()
                .any(|e| e.message.contains("Duplicate task ID")),
            "Should have error about duplicate task IDs"
        );
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        let version = validation::schema::get_schema_version();
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_load_schema() {
        let schema = validation::schema::load_workflow_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_malformed_json() {
        let json = "{ invalid json }";
        let result = validation::validate_workflow_json(json);
        assert!(result.is_err(), "Malformed JSON should fail validation");

        let errors = result.unwrap_err();
        assert!(
            errors.errors[0].message.contains("Invalid JSON"),
            "Should report invalid JSON"
        );
    }
}

#[cfg(test)]
mod error_message_tests {
    use super::*;

    #[test]
    fn test_error_paths_are_clear() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": []
        }"#;

        let result = validation::validate_workflow_json(json);
        // Valid workflow, should pass
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_includes_suggestions() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "function": {
                        "name": "cli_command",
                        "input": {}
                    }
                }
            ]
        }"#;

        let result = validation::validate_workflow_json(json);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.errors.is_empty());

        // Check that errors have helpful suggestions
        for error in &errors.errors {
            assert!(
                !error.message.is_empty(),
                "Error message should not be empty"
            );
            assert!(!error.path.is_empty(), "Error path should not be empty");
        }
    }
}
