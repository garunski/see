//! User input bridge tests
//!
//! Tests for user input conversions between persistence and engine layers

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use persistence::{InputRequestStatus, InputType};
    use serde_json::Value;

    #[test]
    fn test_persistence_to_engine_conversion() {
        use s_e_e_core::UserInputRequest;

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

        // This test verifies that UserInputRequest can be created
        assert_eq!(request.id, "test-id");
        assert_eq!(request.prompt_text, "Enter your name");
        assert!(request.is_pending());
    }

    #[test]
    fn test_input_type_display() {
        use persistence::InputType;

        assert_eq!(InputType::String.to_string(), "string");
        assert_eq!(InputType::Number.to_string(), "number");
        assert_eq!(InputType::Boolean.to_string(), "boolean");
    }

    #[test]
    fn test_input_request_status_display() {
        use persistence::InputRequestStatus;

        assert_eq!(InputRequestStatus::Pending.to_string(), "pending");
        assert_eq!(InputRequestStatus::Fulfilled.to_string(), "fulfilled");
    }

    #[test]
    fn test_user_input_request_validation() {
        use s_e_e_core::UserInputRequest;

        let mut request = UserInputRequest {
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

        // Valid request should pass
        assert!(request.validate().is_ok());
        assert!(request.is_pending());
        assert!(!request.is_fulfilled());

        // Mark as fulfilled
        request.status = InputRequestStatus::Fulfilled;
        request.fulfilled_at = Some(Utc::now());
        request.fulfilled_value = Some("John Doe".to_string());

        // Now should be fulfilled
        assert!(request.is_fulfilled());
        assert!(!request.is_pending());

        // Validation should pass with fulfillment data
        assert!(request.validate().is_ok());
    }
}

