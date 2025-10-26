//! Tests for workflow store operations
//!
//! Tests save_workflow, get_workflow, list_workflows, delete_workflow following Single Responsibility Principle.

use chrono::Utc;
use persistence::{Store, WorkflowDefinition};

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

fn create_test_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("A test workflow".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_save_workflow() {
    let store = create_test_store().await;
    let workflow = create_test_workflow();

    let result = store.save_workflow(&workflow).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_workflow_success() {
    let store = create_test_store().await;
    let workflow = create_test_workflow();

    // Save workflow
    store.save_workflow(&workflow).await.unwrap();

    // Get workflow
    let retrieved = store.get_workflow("test-workflow").await.unwrap();
    assert!(retrieved.is_some());

    let retrieved_workflow = retrieved.unwrap();
    assert_eq!(retrieved_workflow.id, "test-workflow");
    assert_eq!(retrieved_workflow.name, "Test Workflow");
    assert_eq!(
        retrieved_workflow.description,
        Some("A test workflow".to_string())
    );
}

#[tokio::test]
async fn test_get_workflow_not_found() {
    let store = create_test_store().await;

    let retrieved = store.get_workflow("nonexistent").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_list_workflows_empty() {
    let store = create_test_store().await;

    let workflows = store.list_workflows().await.unwrap();
    assert!(workflows.is_empty());
}

#[tokio::test]
async fn test_list_workflows_multiple() {
    let store = create_test_store().await;

    use chrono::Utc;

    // Create multiple workflows with distinct timestamps
    // workflow-2 has later timestamp, so should appear first when sorted DESC
    let workflow1 = WorkflowDefinition {
        id: "workflow-1".to_string(),
        name: "Workflow 1".to_string(),
        content: r#"{"id":"1","name":"1","tasks":[]}"#.to_string(),
        created_at: Utc::now() - chrono::Duration::seconds(10),
        updated_at: Utc::now() - chrono::Duration::seconds(10),
        ..Default::default()
    };

    let workflow2 = WorkflowDefinition {
        id: "workflow-2".to_string(),
        name: "Workflow 2".to_string(),
        content: r#"{"id":"2","name":"2","tasks":[]}"#.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ..Default::default()
    };

    // Save workflows
    store.save_workflow(&workflow1).await.unwrap();
    store.save_workflow(&workflow2).await.unwrap();

    // List workflows
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 2);

    // Check that workflows are ordered by created_at DESC (newest first)
    assert_eq!(workflows[0].id, "workflow-2"); // Newest first
    assert_eq!(workflows[1].id, "workflow-1");
}

#[tokio::test]
async fn test_delete_workflow() {
    let store = create_test_store().await;
    let workflow = create_test_workflow();

    // Save workflow
    store.save_workflow(&workflow).await.unwrap();

    // Verify it exists
    let retrieved = store.get_workflow("test-workflow").await.unwrap();
    assert!(retrieved.is_some());

    // Delete workflow
    let result = store.delete_workflow("test-workflow").await;
    assert!(result.is_ok());

    // Verify it's gone
    let retrieved = store.get_workflow("test-workflow").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_delete_workflow_not_found() {
    let store = create_test_store().await;

    // Delete non-existent workflow should not error
    let result = store.delete_workflow("nonexistent").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_save_workflow_update() {
    let store = create_test_store().await;
    let mut workflow = create_test_workflow();

    // Save initial workflow
    store.save_workflow(&workflow).await.unwrap();

    // Update workflow
    workflow.name = "Updated Workflow".to_string();
    workflow.description = Some("Updated description".to_string());

    // Save updated workflow
    store.save_workflow(&workflow).await.unwrap();

    // Verify update
    let retrieved = store.get_workflow("test-workflow").await.unwrap().unwrap();
    assert_eq!(retrieved.name, "Updated Workflow");
    assert_eq!(
        retrieved.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_workflow_serialization_error() {
    let store = create_test_store().await;

    // Create workflow with invalid JSON content
    let workflow = WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        content: "invalid json".to_string(),
        ..Default::default()
    };

    // This should fail during validation, not during save
    // The save operation itself should succeed, but the content is invalid
    let result = store.save_workflow(&workflow).await;
    assert!(result.is_ok()); // Save succeeds

    // But retrieval should work (we don't validate on retrieval)
    let retrieved = store.get_workflow("test-workflow").await.unwrap();
    assert!(retrieved.is_some());
}
