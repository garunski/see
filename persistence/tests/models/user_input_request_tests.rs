//! Tests for UserInputRequest model
//!
//! Tests serialization, validation, and status management following Single Responsibility Principle.

use persistence::{enums::*, UserInputRequest};
use chrono::Utc;
use serde_json::json;

fn create_test_input_request() -> UserInputRequest {
    UserInputRequest {
        id: "test-request-1".to_string(),
        task_execution_id: "test-task-1".to_string(),
        workflow_execution_id: "test-workflow-1".to_string(),
        prompt_text: "Please enter your name".to_string(),
        input_type: InputType::String,
        required: true,
        default_value: None,
        validation_rules: json!({}),
        status: InputRequestStatus::Pending,
        created_at: Utc::now(),
        fulfilled_at: None,
        fulfilled_value: None,
    }
}

#[test]
fn test_user_input_request_default() {
    let request = UserInputRequest::default();
    
    assert!(!request.id.is_empty());
    assert!(request.task_execution_id.is_empty());
    assert!(request.workflow_execution_id.is_empty());
    assert!(request.prompt_text.is_empty());
    assert_eq!(request.input_type, InputType::String);
    assert!(request.required);
    assert!(request.default_value.is_none());
    assert!(request.validation_rules.is_object());
    assert_eq!(request.status, InputRequestStatus::Pending);
    assert!(request.fulfilled_at.is_none());
    assert!(request.fulfilled_value.is_none());
}

#[test]
fn test_user_input_request_validation_success() {
    let request = create_test_input_request();
    let result = request.validate();
    assert!(result.is_ok());
}

#[test]
fn test_user_input_request_validation_empty_id() {
    let mut request = create_test_input_request();
    request.id = String::new();
    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ID cannot be empty"));
}

#[test]
fn test_user_input_request_validation_empty_task_id() {
    let mut request = create_test_input_request();
    request.task_execution_id = String::new();
    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task execution ID cannot be empty"));
}

#[test]
fn test_user_input_request_validation_empty_workflow_id() {
    let mut request = create_test_input_request();
    request.workflow_execution_id = String::new();
    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Workflow execution ID cannot be empty"));
}

#[test]
fn test_user_input_request_validation_empty_prompt() {
    let mut request = create_test_input_request();
    request.prompt_text = String::new();
    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Prompt text cannot be empty"));
}

#[test]
fn test_user_input_request_validation_fulfilled_without_timestamp() {
    let mut request = create_test_input_request();
    request.status = InputRequestStatus::Fulfilled;
    // Missing fulfilled_at and fulfilled_value
    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("fulfillment timestamp"));
}

#[test]
fn test_user_input_request_validation_fulfilled_with_timestamp_and_value() {
    let mut request = create_test_input_request();
    request.status = InputRequestStatus::Fulfilled;
    request.fulfilled_at = Some(Utc::now());
    request.fulfilled_value = Some("test-value".to_string());
    let result = request.validate();
    assert!(result.is_ok());
}

#[test]
fn test_user_input_request_validation_pending_with_timestamp() {
    let mut request = create_test_input_request();
    request.fulfilled_at = Some(Utc::now());
    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("fulfillment timestamp"));
}

#[test]
fn test_user_input_request_is_fulfilled() {
    let pending_request = create_test_input_request();
    assert!(!pending_request.is_fulfilled());
    
    let mut fulfilled_request = create_test_input_request();
    fulfilled_request.status = InputRequestStatus::Fulfilled;
    fulfilled_request.fulfilled_at = Some(Utc::now());
    fulfilled_request.fulfilled_value = Some("test".to_string());
    assert!(fulfilled_request.is_fulfilled());
}

#[test]
fn test_user_input_request_is_pending() {
    let pending_request = create_test_input_request();
    assert!(pending_request.is_pending());
    
    let mut fulfilled_request = create_test_input_request();
    fulfilled_request.status = InputRequestStatus::Fulfilled;
    fulfilled_request.fulfilled_at = Some(Utc::now());
    fulfilled_request.fulfilled_value = Some("test".to_string());
    assert!(!fulfilled_request.is_pending());
}

#[test]
fn test_user_input_request_serialization() {
    let request = create_test_input_request();
    
    // Test serialization
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test-request-1"));
    assert!(json.contains("test-task-1"));
    assert!(json.contains("test-workflow-1"));
    assert!(json.contains("Please enter your name"));
    assert!(json.contains("string"));
    assert!(json.contains("pending"));
    
    // Test deserialization
    let deserialized: UserInputRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, request.id);
    assert_eq!(deserialized.task_execution_id, request.task_execution_id);
    assert_eq!(deserialized.workflow_execution_id, request.workflow_execution_id);
    assert_eq!(deserialized.prompt_text, request.prompt_text);
    assert_eq!(deserialized.input_type, request.input_type);
    assert_eq!(deserialized.status, request.status);
}

#[test]
fn test_input_type_display() {
    assert_eq!(InputType::String.to_string(), "string");
    assert_eq!(InputType::Number.to_string(), "number");
    assert_eq!(InputType::Boolean.to_string(), "boolean");
}

#[test]
fn test_input_request_status_display() {
    assert_eq!(InputRequestStatus::Pending.to_string(), "pending");
    assert_eq!(InputRequestStatus::Fulfilled.to_string(), "fulfilled");
}

