use s_e_e_core::{
    engine::{pause_workflow, resume_workflow},
    persistence::models::{TaskExecution, WorkflowMetadata, WorkflowStatus},
    types::TaskStatus,
};

#[tokio::test]
async fn test_pause_workflow_function() {
    let execution_id = "test_pause_function";
    let task_id = "test_task";

    // Create a workflow metadata first
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::Running,
        task_ids: vec![task_id.to_string()],
        is_paused: false,
        paused_task_id: None,
    };

    let store = s_e_e_core::get_global_store().unwrap();
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create a task execution
    let task = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: task_id.to_string(),
        task_name: "Test Task".to_string(),
        status: TaskStatus::InProgress,
        logs: vec!["Task started".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task).await.unwrap();

    // Pause the workflow
    let pause_result = pause_workflow(execution_id, task_id).await;
    assert!(pause_result.is_ok());

    // Check that the workflow is paused
    let metadata = store.get_workflow_metadata(execution_id).await.unwrap();
    assert_eq!(metadata.status, WorkflowStatus::WaitingForInput);
    assert!(metadata.is_paused);
    assert_eq!(metadata.paused_task_id, Some(task_id.to_string()));

    // Check that the task is waiting for input
    let tasks = store.get_task_executions(execution_id).await.unwrap();
    let task = tasks.iter().find(|t| t.task_id == task_id).unwrap();
    assert_eq!(task.status, TaskStatus::WaitingForInput);
}

#[tokio::test]
async fn test_resume_workflow_function() {
    let execution_id = "test_resume_function";
    let task_id = "test_task";

    // Create a paused workflow metadata
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::WaitingForInput,
        task_ids: vec![task_id.to_string()],
        is_paused: true,
        paused_task_id: Some(task_id.to_string()),
    };

    let store = s_e_e_core::get_global_store().unwrap();
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create a task execution that's waiting for input
    let task = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: task_id.to_string(),
        task_name: "Test Task".to_string(),
        status: TaskStatus::WaitingForInput,
        logs: vec!["Task paused for input".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task).await.unwrap();

    // Resume the workflow
    let resume_result = resume_workflow(execution_id).await;
    assert!(resume_result.is_ok());

    // Check that the workflow is no longer paused
    let metadata = store.get_workflow_metadata(execution_id).await.unwrap();
    assert_eq!(metadata.status, WorkflowStatus::Running);
    assert!(!metadata.is_paused);
    assert!(metadata.paused_task_id.is_none());
}

#[tokio::test]
async fn test_pause_with_invalid_task_id() {
    let execution_id = "test_invalid_pause";
    let task_id = "nonexistent_task";

    // Create a workflow metadata
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::Running,
        task_ids: vec!["valid_task".to_string()],
        is_paused: false,
        paused_task_id: None,
    };

    let store = s_e_e_core::get_global_store().unwrap();
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Try to pause with invalid task ID
    let pause_result = pause_workflow(execution_id, task_id).await;
    assert!(pause_result.is_err());
}

#[tokio::test]
async fn test_resume_nonexistent_workflow() {
    let execution_id = "nonexistent_workflow";

    // Try to resume a workflow that doesn't exist
    let resume_result = resume_workflow(execution_id).await;
    assert!(resume_result.is_err());
}

#[tokio::test]
async fn test_workflow_status_serialization() {
    // Test that WorkflowStatus::WaitingForInput serializes/deserializes correctly
    let status = WorkflowStatus::WaitingForInput;

    // Serialize to JSON
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"WaitingForInput\"");

    // Deserialize from JSON
    let deserialized: WorkflowStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, WorkflowStatus::WaitingForInput);
}

#[tokio::test]
async fn test_task_status_serialization() {
    // Test that TaskStatus::WaitingForInput serializes/deserializes correctly
    let status = TaskStatus::WaitingForInput;

    // Serialize to JSON
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"waiting-for-input\"");

    // Deserialize from JSON
    let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, TaskStatus::WaitingForInput);
}

#[tokio::test]
async fn test_workflow_metadata_serialization() {
    // Test that WorkflowMetadata with pause fields serializes/deserializes correctly
    let metadata = WorkflowMetadata {
        id: "serialization_test".to_string(),
        workflow_name: "Serialization Test".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::WaitingForInput,
        task_ids: vec!["task1".to_string()],
        is_paused: true,
        paused_task_id: Some("task1".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("is_paused"));
    assert!(json.contains("paused_task_id"));

    // Deserialize from JSON
    let deserialized: WorkflowMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, metadata.id);
    assert_eq!(deserialized.is_paused, metadata.is_paused);
    assert_eq!(deserialized.paused_task_id, metadata.paused_task_id);
    assert_eq!(deserialized.status, WorkflowStatus::WaitingForInput);
}
