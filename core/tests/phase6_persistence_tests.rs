use s_e_e_core::engine::execute::{pause_workflow, resume_task, resume_workflow};
use s_e_e_core::persistence::models::{TaskExecution, WorkflowMetadata, WorkflowStatus};
use s_e_e_core::persistence::store::{AuditStore, RedbStore};
use s_e_e_core::types::TaskStatus;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper function to create a test database
fn create_test_store() -> (RedbStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.audit.redb");
    let store = RedbStore::new(db_path).unwrap();
    (store, temp_dir)
}

/// Helper function to get the global store for testing
fn get_global_store() -> Result<Arc<dyn AuditStore + Send + Sync>, Box<dyn std::error::Error>> {
    Ok(s_e_e_core::get_global_store()?)
}

/// Helper function to create test workflow metadata
fn create_test_workflow_metadata(
    id: &str,
    status: WorkflowStatus,
    is_paused: bool,
    paused_task_id: Option<String>,
) -> WorkflowMetadata {
    WorkflowMetadata {
        id: id.to_string(),
        workflow_name: "Test Workflow".to_string(),
        start_timestamp: "2024-12-19T10:00:00Z".to_string(),
        end_timestamp: None,
        status,
        task_ids: vec!["task1".to_string()],
        is_paused,
        paused_task_id,
    }
}

/// Helper function to create test task execution
fn create_test_task_execution(
    execution_id: &str,
    task_id: &str,
    status: TaskStatus,
) -> TaskExecution {
    TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: task_id.to_string(),
        task_name: task_id.to_string(),
        status,
        logs: vec!["Test log".to_string()],
        start_timestamp: "2024-12-19T10:00:00Z".to_string(),
        end_timestamp: "".to_string(),
    }
}

#[tokio::test]
async fn test_workflow_metadata_serialization_with_new_fields() {
    // Test serialization with new fields
    let metadata = create_test_workflow_metadata(
        "test-1",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task1".to_string()),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"is_paused\":true"));
    assert!(json.contains("\"paused_task_id\":\"task1\""));

    // Deserialize back
    let deserialized: WorkflowMetadata = serde_json::from_str(&json).unwrap();
    assert!(deserialized.is_paused);
    assert_eq!(deserialized.paused_task_id, Some("task1".to_string()));
}

#[tokio::test]
async fn test_workflow_metadata_serialization_with_defaults() {
    // Test serialization with default values
    let metadata = create_test_workflow_metadata("test-2", WorkflowStatus::Running, false, None);

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"is_paused\":false"));
    assert!(json.contains("\"paused_task_id\":null"));

    // Deserialize back
    let deserialized: WorkflowMetadata = serde_json::from_str(&json).unwrap();
    assert!(!deserialized.is_paused);
    assert_eq!(deserialized.paused_task_id, None);
}

#[tokio::test]
async fn test_mark_workflow_paused() {
    let (store, _temp_dir) = create_test_store();

    // Create initial workflow metadata
    let metadata =
        create_test_workflow_metadata("test-pause", WorkflowStatus::Running, false, None);
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Mark workflow as paused
    store
        .mark_workflow_paused("test-pause", "task1")
        .await
        .unwrap();

    // Verify the workflow is marked as paused
    let updated_metadata = store.get_workflow_metadata("test-pause").await.unwrap();
    assert!(updated_metadata.is_paused);
    assert_eq!(updated_metadata.paused_task_id, Some("task1".to_string()));
    assert_eq!(updated_metadata.status, WorkflowStatus::WaitingForInput);
}

#[tokio::test]
async fn test_mark_workflow_resumed() {
    let (store, _temp_dir) = create_test_store();

    // Create paused workflow metadata
    let metadata = create_test_workflow_metadata(
        "test-resume",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task1".to_string()),
    );
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Mark workflow as resumed
    store.mark_workflow_resumed("test-resume").await.unwrap();

    // Verify the workflow is marked as resumed
    let updated_metadata = store.get_workflow_metadata("test-resume").await.unwrap();
    assert!(!updated_metadata.is_paused);
    assert_eq!(updated_metadata.paused_task_id, None);
    assert_eq!(updated_metadata.status, WorkflowStatus::Running);
}

#[tokio::test]
async fn test_get_paused_workflows() {
    let (store, _temp_dir) = create_test_store();

    // Create multiple workflows with different statuses
    let running_workflow =
        create_test_workflow_metadata("running-workflow", WorkflowStatus::Running, false, None);
    let paused_workflow1 = create_test_workflow_metadata(
        "paused-workflow-1",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task1".to_string()),
    );
    let paused_workflow2 = create_test_workflow_metadata(
        "paused-workflow-2",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task2".to_string()),
    );

    store
        .save_workflow_metadata(&running_workflow)
        .await
        .unwrap();
    store
        .save_workflow_metadata(&paused_workflow1)
        .await
        .unwrap();
    store
        .save_workflow_metadata(&paused_workflow2)
        .await
        .unwrap();

    // Get paused workflows
    let paused_workflows = store.get_paused_workflows().await.unwrap();

    // Should only return the paused workflows
    assert_eq!(paused_workflows.len(), 2);
    let paused_ids: Vec<&String> = paused_workflows.iter().map(|w| &w.id).collect();
    assert!(paused_ids.contains(&&"paused-workflow-1".to_string()));
    assert!(paused_ids.contains(&&"paused-workflow-2".to_string()));
    assert!(!paused_ids.contains(&&"running-workflow".to_string()));
}

#[tokio::test]
async fn test_pause_workflow_success() {
    let store = get_global_store().unwrap();

    // Create workflow metadata
    let metadata =
        create_test_workflow_metadata("test-pause-workflow", WorkflowStatus::Running, false, None);
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create task execution
    let task_exec =
        create_test_task_execution("test-pause-workflow", "task1", TaskStatus::InProgress);
    store.save_task_execution(&task_exec).await.unwrap();

    // Pause the workflow
    pause_workflow("test-pause-workflow", "task1")
        .await
        .unwrap();

    // Verify task status is updated
    let task_executions = store
        .get_task_executions("test-pause-workflow")
        .await
        .unwrap();
    let task = task_executions
        .iter()
        .find(|t| t.task_id == "task1")
        .unwrap();
    assert_eq!(task.status, TaskStatus::WaitingForInput);

    // Verify workflow metadata is updated
    let updated_metadata = store
        .get_workflow_metadata("test-pause-workflow")
        .await
        .unwrap();
    assert!(updated_metadata.is_paused);
    assert_eq!(updated_metadata.paused_task_id, Some("task1".to_string()));
    assert_eq!(updated_metadata.status, WorkflowStatus::WaitingForInput);
}

#[tokio::test]
async fn test_pause_workflow_task_not_found() {
    let store = get_global_store().unwrap();

    // Create workflow metadata but no task execution
    let metadata =
        create_test_workflow_metadata("test-pause-not-found", WorkflowStatus::Running, false, None);
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Try to pause non-existent task
    let result = pause_workflow("test-pause-not-found", "nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_resume_workflow_uses_persistence_methods() {
    let store = get_global_store().unwrap();

    // Create paused workflow metadata
    let metadata = create_test_workflow_metadata(
        "test-resume-workflow",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task1".to_string()),
    );
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create waiting task execution
    let task_exec =
        create_test_task_execution("test-resume-workflow", "task1", TaskStatus::WaitingForInput);
    store.save_task_execution(&task_exec).await.unwrap();

    // Resume the workflow
    resume_workflow("test-resume-workflow").await.unwrap();

    // Verify task status is updated
    let task_executions = store
        .get_task_executions("test-resume-workflow")
        .await
        .unwrap();
    let task = task_executions
        .iter()
        .find(|t| t.task_id == "task1")
        .unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);

    // Verify workflow metadata is updated
    let updated_metadata = store
        .get_workflow_metadata("test-resume-workflow")
        .await
        .unwrap();
    assert!(!updated_metadata.is_paused);
    assert_eq!(updated_metadata.paused_task_id, None);
    assert_eq!(updated_metadata.status, WorkflowStatus::Running);
}

#[tokio::test]
async fn test_resume_task_uses_persistence_methods() {
    let store = get_global_store().unwrap();

    // Create paused workflow metadata
    let metadata = create_test_workflow_metadata(
        "test-resume-task",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task1".to_string()),
    );
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create waiting task execution
    let task_exec =
        create_test_task_execution("test-resume-task", "task1", TaskStatus::WaitingForInput);
    store.save_task_execution(&task_exec).await.unwrap();

    // Resume the specific task
    resume_task("test-resume-task", "task1").await.unwrap();

    // Verify task status is updated
    let task_executions = store.get_task_executions("test-resume-task").await.unwrap();
    let task = task_executions
        .iter()
        .find(|t| t.task_id == "task1")
        .unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);

    // Verify workflow metadata is updated (since no more waiting tasks)
    let updated_metadata = store
        .get_workflow_metadata("test-resume-task")
        .await
        .unwrap();
    assert!(!updated_metadata.is_paused);
    assert_eq!(updated_metadata.paused_task_id, None);
    assert_eq!(updated_metadata.status, WorkflowStatus::Running);
}

#[tokio::test]
async fn test_resume_task_multiple_waiting_tasks() {
    let store = get_global_store().unwrap();

    // Create paused workflow metadata
    let metadata = create_test_workflow_metadata(
        "test-resume-multiple",
        WorkflowStatus::WaitingForInput,
        true,
        Some("task1".to_string()),
    );
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create multiple waiting task executions
    let task1 =
        create_test_task_execution("test-resume-multiple", "task1", TaskStatus::WaitingForInput);
    let task2 =
        create_test_task_execution("test-resume-multiple", "task2", TaskStatus::WaitingForInput);
    store.save_task_execution(&task1).await.unwrap();
    store.save_task_execution(&task2).await.unwrap();

    // Resume only task1
    resume_task("test-resume-multiple", "task1").await.unwrap();

    // Verify task1 status is updated
    let task_executions = store
        .get_task_executions("test-resume-multiple")
        .await
        .unwrap();
    let task1_updated = task_executions
        .iter()
        .find(|t| t.task_id == "task1")
        .unwrap();
    assert_eq!(task1_updated.status, TaskStatus::InProgress);

    // Verify task2 is still waiting
    let task2_updated = task_executions
        .iter()
        .find(|t| t.task_id == "task2")
        .unwrap();
    assert_eq!(task2_updated.status, TaskStatus::WaitingForInput);

    // Verify workflow metadata is still paused (since task2 is still waiting)
    let updated_metadata = store
        .get_workflow_metadata("test-resume-multiple")
        .await
        .unwrap();
    assert!(updated_metadata.is_paused);
    assert_eq!(updated_metadata.status, WorkflowStatus::WaitingForInput);
}

#[tokio::test]
async fn test_full_pause_resume_persistence_cycle() {
    let store = get_global_store().unwrap();

    // Create initial workflow
    let metadata =
        create_test_workflow_metadata("test-cycle", WorkflowStatus::Running, false, None);
    store.save_workflow_metadata(&metadata).await.unwrap();

    let task_exec = create_test_task_execution("test-cycle", "task1", TaskStatus::InProgress);
    store.save_task_execution(&task_exec).await.unwrap();

    // 1. Pause the workflow
    pause_workflow("test-cycle", "task1").await.unwrap();

    // Verify paused state
    let paused_metadata = store.get_workflow_metadata("test-cycle").await.unwrap();
    assert!(paused_metadata.is_paused);
    assert_eq!(paused_metadata.paused_task_id, Some("task1".to_string()));
    assert_eq!(paused_metadata.status, WorkflowStatus::WaitingForInput);

    let paused_tasks = store.get_task_executions("test-cycle").await.unwrap();
    let paused_task = paused_tasks.iter().find(|t| t.task_id == "task1").unwrap();
    assert_eq!(paused_task.status, TaskStatus::WaitingForInput);

    // 2. Resume the workflow
    resume_workflow("test-cycle").await.unwrap();

    // Verify resumed state
    let resumed_metadata = store.get_workflow_metadata("test-cycle").await.unwrap();
    assert!(!resumed_metadata.is_paused);
    assert_eq!(resumed_metadata.paused_task_id, None);
    assert_eq!(resumed_metadata.status, WorkflowStatus::Running);

    let resumed_tasks = store.get_task_executions("test-cycle").await.unwrap();
    let resumed_task = resumed_tasks.iter().find(|t| t.task_id == "task1").unwrap();
    assert_eq!(resumed_task.status, TaskStatus::InProgress);
}

#[tokio::test]
async fn test_error_handling_workflow_not_found() {
    let _store = get_global_store().unwrap();

    // Try to pause non-existent workflow
    let result = pause_workflow("nonexistent", "task1").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    // Try to resume non-existent workflow
    let result = resume_workflow("nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    // Try to resume task in non-existent workflow
    let result = resume_task("nonexistent", "task1").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_error_handling_wrong_status() {
    let store = get_global_store().unwrap();

    // Create workflow that's not waiting for input
    let metadata =
        create_test_workflow_metadata("test-wrong-status", WorkflowStatus::Complete, false, None);
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Try to resume workflow that's not waiting
    let result = resume_workflow("test-wrong-status").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not waiting for input"));

    // Create task that's not waiting for input
    let task_exec = create_test_task_execution("test-wrong-status", "task1", TaskStatus::Complete);
    store.save_task_execution(&task_exec).await.unwrap();

    // Try to resume task that's not waiting
    let result = resume_task("test-wrong-status", "task1").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not waiting for input"));
}
