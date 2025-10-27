//! Integration tests for workflow execution
//! Tests the full execution flow using the global store

use s_e_e_core::store_singleton::{cleanup_test_db, init_test_store};
use s_e_e_core::{execute_workflow_by_id, get_global_store, WorkflowDefinition};
use uuid::Uuid;

#[tokio::test]
async fn test_workflow_execution_preserves_logs() {
    // Clean up any existing test database
    let _ = cleanup_test_db();

    // Initialize test store
    init_test_store().await.unwrap();

    // Get the global store
    let store = get_global_store().unwrap();

    // Create and save workflow
    let workflow_def = WorkflowDefinition {
        id: Uuid::new_v4().to_string(),
        name: "Test Workflow".to_string(),
        description: Some("Test workflow for log preservation".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[{"id":"task1","name":"Echo","function":{"name":"cli_command","input":{"command":"echo","args":["TestOutput"]}},"next_tasks":[]}]}"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    store.save_workflow(&workflow_def).await.unwrap();

    // Execute workflow
    let workflow_result = execute_workflow_by_id(&workflow_def.id, None)
        .await
        .unwrap();

    // Verify logs were captured
    assert!(!workflow_result.per_task_logs.is_empty());
    assert!(workflow_result.per_task_logs.contains_key("task1"));
    let logs = &workflow_result.per_task_logs["task1"];
    assert!(logs.iter().any(|log| log.contains("TestOutput")));

    // Load execution from database
    let execution = store
        .get_workflow_with_tasks(&workflow_result.execution_id)
        .await
        .unwrap();

    // Verify logs are in database
    assert!(!execution.per_task_logs.is_empty());
    assert!(execution.tasks[0].output.is_some());
    assert!(execution.tasks[0]
        .output
        .as_ref()
        .unwrap()
        .contains("TestOutput"));

    // Clean up test database
    cleanup_test_db().unwrap();
}
