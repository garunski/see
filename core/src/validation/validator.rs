//! JSON Schema validator for workflow definitions

use crate::validation::types::{ValidationError, ValidationErrors, WorkflowSchema};
use jsonschema::JSONSchema;
use serde_json::Value;

/// Validate a workflow JSON string against the schema
///
/// # Arguments
/// * `json_str` - The workflow JSON as a string
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(ValidationErrors)` with detailed error information if validation fails
pub fn validate_workflow_json(json_str: &str) -> Result<(), ValidationErrors> {
    // First, validate that the JSON is well-formed
    let workflow_json: Value = serde_json::from_str(json_str).map_err(|e| ValidationErrors {
        errors: vec![ValidationError {
            path: "/".to_string(),
            message: format!("Invalid JSON: {}", e),
            expected: Some("valid JSON".to_string()),
            suggestions: vec![
                "Check JSON syntax (commas, brackets, quotes)".to_string(),
                "Ensure all strings are properly quoted".to_string(),
                "Verify that all brackets and braces are balanced".to_string(),
            ],
        }],
    })?;

    // Load manual schema
    let schema = super::schema::load_workflow_schema();

    // Compile the schema
    let compiled = JSONSchema::compile(&schema).map_err(|e| ValidationErrors {
        errors: vec![ValidationError {
            path: "/".to_string(),
            message: format!("Failed to compile schema: {}", e),
            expected: None,
            suggestions: vec!["This is an internal error".to_string()],
        }],
    })?;

    // Validate - need to collect errors to avoid lifetime issues
    let validation_result = compiled.validate(&workflow_json);

    // If basic JSON Schema validation passes, do additional custom validations
    if let Ok(()) = validation_result {
        let workflow: WorkflowSchema =
            serde_json::from_value(workflow_json.clone()).map_err(|e| ValidationErrors {
                errors: vec![ValidationError {
                    path: "/".to_string(),
                    message: format!("Failed to deserialize workflow: {}", e),
                    expected: None,
                    suggestions: vec![
                        "Check that the JSON matches the expected structure".to_string()
                    ],
                }],
            })?;

        // Check for duplicate task IDs
        super::types::validate_no_duplicate_task_ids(&workflow).map_err(|msg| {
            ValidationErrors {
                errors: vec![ValidationError {
                    path: "tasks".to_string(),
                    message: msg,
                    expected: None,
                    suggestions: vec![
                        "Ensure all task IDs are unique".to_string(),
                        "Check nested next_tasks arrays for duplicates".to_string(),
                    ],
                }],
            }
        })?;

        Ok(())
    } else {
        // Convert validation errors to our format
        let errors: Vec<_> = validation_result.unwrap_err().collect();
        let validation_errors: Vec<ValidationError> = errors
            .into_iter()
            .map(|error| {
                let instance_path = error.instance_path.to_string();

                let message = if !error.to_string().is_empty() {
                    error.to_string()
                } else {
                    format!("Validation failed at {}", instance_path)
                };

                // Extract more detailed information
                let mut expected = None;
                let mut suggestions = Vec::new();

                // Provide context-specific suggestions based on error path
                let path_lower = instance_path.to_lowercase();
                if path_lower.contains("function") {
                    suggestions.push("Ensure 'function' field contains 'name' and 'input' fields".to_string());
                    if path_lower.contains("cli_command") {
                        suggestions.push("CLI commands require 'input.command' (string) and optionally 'input.args' (array of strings)".to_string());
                    } else if path_lower.contains("user_input") {
                        suggestions.push("User input requires 'input.prompt' (string) and 'input.input_type' (string)".to_string());
                    } else if path_lower.contains("cursor_agent") {
                        suggestions.push("Cursor agent requires 'input.prompt' (string)".to_string());
                    }
                } else if path_lower.contains("/id") {
                    suggestions.push("IDs must be unique non-empty strings".to_string());
                    expected = Some("non-empty string".to_string());
                } else if path_lower.contains("/name") {
                    suggestions.push("Names should be descriptive non-empty strings".to_string());
                    expected = Some("non-empty string".to_string());
                }

                ValidationError {
                    path: if instance_path.is_empty() || instance_path == "/" {
                        "root".to_string()
                    } else {
                        instance_path
                    },
                    message,
                    expected,
                    suggestions,
                }
            })
            .collect();

        Err(ValidationErrors {
            errors: validation_errors,
        })
    }
}

/// Validate a workflow JSON string and return a simple error message
///
/// This is a convenience function for cases where detailed error information
/// is not needed.
///
/// # Arguments
/// * `json_str` - The workflow JSON as a string
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(String)` with a summary error message
pub fn validate_workflow_json_simple(json_str: &str) -> Result<(), String> {
    validate_workflow_json(json_str).map_err(|errors| errors.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_workflow() {
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
                            "args": ["hello"]
                        }
                    },
                    "next_tasks": []
                }
            ]
        }"#;

        assert!(validate_workflow_json(json).is_ok());
    }

    #[test]
    fn test_validate_missing_required_fields() {
        let json = r#"{
            "id": "test"
        }"#;

        assert!(validate_workflow_json(json).is_err());
    }

    #[test]
    fn test_validate_invalid_json() {
        let json = "{ invalid json }";
        assert!(validate_workflow_json(json).is_err());
    }

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
                            },
                            "next_tasks": []
                        }
                    ]
                }
            ]
        }"#;

        let result = validate_workflow_json(json);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .errors
            .iter()
            .any(|e| e.message.contains("Duplicate task ID")));
    }

    #[test]
    fn test_validate_missing_function() {
        let json = r#"{
            "id": "test",
            "name": "Test Workflow",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1"
                }
            ]
        }"#;

        assert!(validate_workflow_json(json).is_err());
    }
}
