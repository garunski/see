//! User input bridge conversions
//!
//! This file contains ONLY user input conversions between persistence and engine layers.

use persistence::{InputRequestStatus, InputType, UserInputRequest};
use serde_json::Value;

/// Convert persistence UserInputRequest to engine-compatible format
/// 
/// Since the engine doesn't have a separate UserInputRequest type,
/// we convert to a format that can be used by the engine layer.
pub fn persistence_to_engine_input_request(
    input: &UserInputRequest,
) -> Result<Value, String> {
    let engine_value = serde_json::json!({
        "id": input.id,
        "task_execution_id": input.task_execution_id,
        "workflow_execution_id": input.workflow_execution_id,
        "prompt_text": input.prompt_text,
        "input_type": input.input_type.to_string(),
        "required": input.required,
        "default_value": input.default_value,
        "validation_rules": input.validation_rules,
        "status": input.status.to_string(),
        "created_at": input.created_at.to_rfc3339(),
        "fulfilled_at": input.fulfilled_at.map(|dt| dt.to_rfc3339()),
        "fulfilled_value": input.fulfilled_value,
    });
    
    Ok(engine_value)
}

/// Convert engine-compatible value to persistence UserInputRequest
pub fn engine_to_persistence_input_request(
    value: &Value,
) -> Result<UserInputRequest, String> {
    let input: UserInputRequest = serde_json::from_value(value.clone())
        .map_err(|e| format!("Failed to deserialize UserInputRequest: {}", e))?;
    
    Ok(input)
}

/// Convert InputType enum from string
pub fn parse_input_type(type_str: &str) -> Result<InputType, String> {
    match type_str {
        "string" => Ok(InputType::String),
        "number" => Ok(InputType::Number),
        "boolean" => Ok(InputType::Boolean),
        _ => Err(format!("Unknown input type: {}", type_str)),
    }
}

/// Convert InputRequestStatus enum from string
pub fn parse_input_request_status(status_str: &str) -> Result<InputRequestStatus, String> {
    match status_str {
        "pending" => Ok(InputRequestStatus::Pending),
        "fulfilled" => Ok(InputRequestStatus::Fulfilled),
        _ => Err(format!("Unknown input request status: {}", status_str)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_persistence_to_engine_conversion() {
        let request = UserInputRequest {
            id: "test-id".to_string(),
            task_execution_id: "task-123".to_string(),
            workflow_execution_id: "workflow-456".to_string(),
            prompt_text: "Enter your name".to_string(),
            input_type: InputType::String,
            required: true,
            default_value: None,
            validation_rules: Value::Object(serde_json::Map::new()),
            status: InputRequestStatus::Pending,
            created_at: Utc::now(),
            fulfilled_at: None,
            fulfilled_value: None,
        };

        let result = persistence_to_engine_input_request(&request).unwrap();
        assert_eq!(result["id"], "test-id");
        assert_eq!(result["input_type"], "string");
        assert_eq!(result["status"], "pending");
    }

    #[test]
    fn test_parse_input_types() {
        assert!(matches!(parse_input_type("string"), Ok(InputType::String)));
        assert!(matches!(parse_input_type("number"), Ok(InputType::Number)));
        assert!(matches!(parse_input_type("boolean"), Ok(InputType::Boolean)));
        assert!(parse_input_type("invalid").is_err());
    }

    #[test]
    fn test_parse_input_request_status() {
        assert!(matches!(
            parse_input_request_status("pending"),
            Ok(InputRequestStatus::Pending)
        ));
        assert!(matches!(
            parse_input_request_status("fulfilled"),
            Ok(InputRequestStatus::Fulfilled)
        ));
        assert!(parse_input_request_status("invalid").is_err());
    }
}

