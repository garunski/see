//! Tests for user input store operations
//!
//! Tests CRUD operations for UserInputRequest following Single Responsibility Principle.

use s_e_e_persistence::{enums::*, Store, UserInputRequest};
use chrono::Utc;
use serde_json::json;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

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

#[tokio::test]
async fn test_save_input_request() {
    let store = create_test_store().await;
    let request = create_test_input_request();
    
    let result = store.save_input_request(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_input_request_not_found() {
    let store = create_test_store().await;
    
    let result = store.get_input_request("non-existent-id").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_input_request_found() {
    let store = create_test_store().await;
    let request = create_test_input_request();
    
    // Save request
    store.save_input_request(&request).await.unwrap();
    
    // Get request
    let retrieved = store.get_input_request("test-request-1").await.unwrap();
    assert!(retrieved.is_some());
    
    let retrieved_request = retrieved.unwrap();
    assert_eq!(retrieved_request.id, request.id);
    assert_eq!(retrieved_request.task_execution_id, request.task_execution_id);
    assert_eq!(retrieved_request.workflow_execution_id, request.workflow_execution_id);
    assert_eq!(retrieved_request.prompt_text, request.prompt_text);
    assert_eq!(retrieved_request.input_type, request.input_type);
    assert_eq!(retrieved_request.status, request.status);
}

#[tokio::test]
async fn test_get_input_request_by_task() {
    let store = create_test_store().await;
    let request = create_test_input_request();
    
    // Save request
    store.save_input_request(&request).await.unwrap();
    
    // Get request by task ID
    let retrieved = store.get_input_request_by_task("test-task-1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "test-request-1");
}

#[tokio::test]
async fn test_get_input_request_by_task_not_found() {
    let store = create_test_store().await;
    
    let result = store.get_input_request_by_task("non-existent-task").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_pending_inputs_for_workflow() {
    let store = create_test_store().await;
    
    // Create multiple requests for same workflow
    let request1 = UserInputRequest {
        id: "request-1".to_string(),
        task_execution_id: "task-1".to_string(),
        workflow_execution_id: "workflow-1".to_string(),
        status: InputRequestStatus::Pending,
        ..create_test_input_request()
    };
    
    let request2 = UserInputRequest {
        id: "request-2".to_string(),
        task_execution_id: "task-2".to_string(),
        workflow_execution_id: "workflow-1".to_string(),
        status: InputRequestStatus::Pending,
        ..create_test_input_request()
    };
    
    let request3 = UserInputRequest {
        id: "request-3".to_string(),
        task_execution_id: "task-3".to_string(),
        workflow_execution_id: "workflow-1".to_string(),
        status: InputRequestStatus::Fulfilled,
        fulfilled_at: Some(Utc::now()),
        fulfilled_value: Some("value".to_string()),
        ..create_test_input_request()
    };
    
    let request4 = UserInputRequest {
        id: "request-4".to_string(),
        task_execution_id: "task-4".to_string(),
        workflow_execution_id: "workflow-2".to_string(), // Different workflow
        status: InputRequestStatus::Pending,
        ..create_test_input_request()
    };
    
    // Save all requests
    store.save_input_request(&request1).await.unwrap();
    store.save_input_request(&request2).await.unwrap();
    store.save_input_request(&request3).await.unwrap();
    store.save_input_request(&request4).await.unwrap();
    
    // Get pending inputs for workflow-1
    let pending = store.get_pending_inputs_for_workflow("workflow-1").await.unwrap();
    assert_eq!(pending.len(), 2);
    
    let request_ids: Vec<&str> = pending.iter().map(|r| r.id.as_str()).collect();
    assert!(request_ids.contains(&"request-1"));
    assert!(request_ids.contains(&"request-2"));
    assert!(!request_ids.contains(&"request-3")); // Fulfilled
    assert!(!request_ids.contains(&"request-4")); // Different workflow
}

#[tokio::test]
async fn test_get_all_pending_inputs() {
    let store = create_test_store().await;
    
    let request1 = UserInputRequest {
        status: InputRequestStatus::Pending,
        ..create_test_input_request()
    };
    
    let request2 = UserInputRequest {
        id: "request-2".to_string(),
        task_execution_id: "task-2".to_string(),
        status: InputRequestStatus::Fulfilled,
        fulfilled_at: Some(Utc::now()),
        fulfilled_value: Some("value".to_string()),
        ..create_test_input_request()
    };
    
    store.save_input_request(&request1).await.unwrap();
    store.save_input_request(&request2).await.unwrap();
    
    let all_pending = store.get_all_pending_inputs().await.unwrap();
    assert_eq!(all_pending.len(), 1);
    assert_eq!(all_pending[0].id, "test-request-1");
}

#[tokio::test]
async fn test_fulfill_input_request() {
    let store = create_test_store().await;
    let request = create_test_input_request();
    
    // Save pending request
    store.save_input_request(&request).await.unwrap();
    
    // Fulfill request
    let result = store.fulfill_input_request("test-request-1", "user-input-value".to_string()).await;
    assert!(result.is_ok());
    
    // Verify status change
    let retrieved = store.get_input_request("test-request-1").await.unwrap().unwrap();
    assert!(matches!(retrieved.status, InputRequestStatus::Fulfilled));
    assert!(retrieved.fulfilled_at.is_some());
    assert_eq!(retrieved.fulfilled_value, Some("user-input-value".to_string()));
}

#[tokio::test]
async fn test_fulfill_input_request_not_found() {
    let store = create_test_store().await;
    
    let result = store.fulfill_input_request("non-existent", "value".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_input_request() {
    let store = create_test_store().await;
    let request = create_test_input_request();
    
    // Save request
    store.save_input_request(&request).await.unwrap();
    
    // Verify request exists
    let retrieved = store.get_input_request("test-request-1").await.unwrap();
    assert!(retrieved.is_some());
    
    // Delete request
    let result = store.delete_input_request("test-request-1").await;
    assert!(result.is_ok());
    
    // Verify request deleted
    let retrieved = store.get_input_request("test-request-1").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_input_request_serialization_round_trip() {
    let store = create_test_store().await;
    let request = create_test_input_request();
    
    // Save request
    store.save_input_request(&request).await.unwrap();
    
    // Retrieve and verify all fields
    let retrieved = store.get_input_request("test-request-1").await.unwrap().unwrap();
    assert_eq!(retrieved.id, request.id);
    assert_eq!(retrieved.task_execution_id, request.task_execution_id);
    assert_eq!(retrieved.workflow_execution_id, request.workflow_execution_id);
    assert_eq!(retrieved.prompt_text, request.prompt_text);
    assert_eq!(retrieved.input_type, request.input_type);
    assert_eq!(retrieved.required, request.required);
    assert_eq!(retrieved.status, request.status);
}

