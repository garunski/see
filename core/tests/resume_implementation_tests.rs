use s_e_e_core::{
    engine::{resume_task, resume_workflow},
    persistence::models::{TaskExecution, WorkflowMetadata, WorkflowStatus},
    types::TaskStatus,
};

/// Test helper to create test data in database
async fn setup_test_workflow_waiting(execution_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let store = s_e_e_core::get_global_store()?;

    // Create workflow metadata with WaitingForInput status
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Waiting Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::WaitingForInput,
        task_ids: vec!["task1".to_string(), "task2".to_string()],
    };
    store.save_workflow_metadata(&metadata).await?;

    // Create task executions with WaitingForInput status
    let task1 = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: "task1".to_string(),
        task_name: "Test Task 1".to_string(),
        status: TaskStatus::WaitingForInput,
        logs: vec!["Task paused for input".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task1).await?;

    let task2 = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: "task2".to_string(),
        task_name: "Test Task 2".to_string(),
        status: TaskStatus::WaitingForInput,
        logs: vec!["Task paused for input".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task2).await?;

    Ok(())
}

/// Test helper to create test data with running workflow
async fn setup_test_workflow_running(execution_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let store = s_e_e_core::get_global_store()?;

    // Create workflow metadata with Running status
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Running Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::Running,
        task_ids: vec!["task1".to_string()],
    };
    store.save_workflow_metadata(&metadata).await?;

    Ok(())
}

/// Test helper to verify task status in database
async fn verify_task_status(
    execution_id: &str,
    task_id: &str,
    expected_status: TaskStatus,
) -> Result<bool, Box<dyn std::error::Error>> {
    let store = s_e_e_core::get_global_store()?;
    let tasks = store.get_task_executions(execution_id).await?;

    let task = tasks
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or("Task not found")?;

    Ok(task.status == expected_status)
}

/// Test helper to verify workflow status in database
async fn verify_workflow_status(
    execution_id: &str,
    expected_status: WorkflowStatus,
) -> Result<bool, Box<dyn std::error::Error>> {
    let store = s_e_e_core::get_global_store()?;
    let metadata = store.get_workflow_metadata(execution_id).await?;

    Ok(metadata.status == expected_status)
}

#[tokio::test]
async fn test_resume_workflow_success() {
    let execution_id = "test-resume-workflow-success";

    // Setup test data
    setup_test_workflow_waiting(execution_id).await.unwrap();

    // Verify initial state
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::WaitingForInput)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task1", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task2", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );

    // Resume workflow
    let result = resume_workflow(execution_id).await;
    assert!(result.is_ok(), "resume_workflow should succeed");

    // Verify final state
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::Running)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task1", TaskStatus::InProgress)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task2", TaskStatus::InProgress)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_resume_workflow_not_found() {
    let execution_id = "test-resume-workflow-not-found";

    // Try to resume non-existent workflow
    let result = resume_workflow(execution_id).await;
    assert!(
        result.is_err(),
        "resume_workflow should fail for non-existent workflow"
    );

    // Verify error type
    match result.unwrap_err() {
        s_e_e_core::errors::CoreError::Dataflow(_) => {
            // Expected error type
        }
        other => panic!("Expected Dataflow error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_resume_workflow_wrong_status() {
    let execution_id = "test-resume-workflow-wrong-status";

    // Setup workflow with Running status (not WaitingForInput)
    setup_test_workflow_running(execution_id).await.unwrap();

    // Try to resume workflow that's not waiting
    let result = resume_workflow(execution_id).await;
    assert!(
        result.is_err(),
        "resume_workflow should fail for non-waiting workflow"
    );

    // Verify error type
    match result.unwrap_err() {
        s_e_e_core::errors::CoreError::Validation(_) => {
            // Expected error type
        }
        other => panic!("Expected Validation error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_resume_task_success() {
    let execution_id = "test-resume-task-success";
    let store = s_e_e_core::get_global_store().unwrap();

    // Clean up any existing data
    let _ = store.delete_workflow_metadata_and_tasks(execution_id).await;

    // Setup workflow with only one waiting task
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Single Task Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::WaitingForInput,
        task_ids: vec!["task1".to_string()],
    };
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create only one task with WaitingForInput status
    let task1 = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: "task1".to_string(),
        task_name: "Test Task 1".to_string(),
        status: TaskStatus::WaitingForInput,
        logs: vec!["Task paused for input".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task1).await.unwrap();

    // Verify initial state
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::WaitingForInput)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task1", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );

    // Resume specific task
    let result = resume_task(execution_id, "task1").await;
    assert!(result.is_ok(), "resume_task should succeed");

    // Verify task status changed
    assert!(
        verify_task_status(execution_id, "task1", TaskStatus::InProgress)
            .await
            .unwrap()
    );

    // Verify workflow status changed to Running (no more waiting tasks)
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::Running)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_resume_task_not_found() {
    let execution_id = "test-resume-task-not-found";

    // Setup test data
    setup_test_workflow_waiting(execution_id).await.unwrap();

    // Try to resume non-existent task
    let result = resume_task(execution_id, "nonexistent-task").await;
    assert!(
        result.is_err(),
        "resume_task should fail for non-existent task"
    );

    // Verify error type
    match result.unwrap_err() {
        s_e_e_core::errors::CoreError::Validation(_) => {
            // Expected error type
        }
        other => panic!("Expected Validation error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_resume_task_wrong_status() {
    let execution_id = "test-resume-task-wrong-status";
    let store = s_e_e_core::get_global_store().unwrap();

    // Setup workflow
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::Running,
        task_ids: vec!["task1".to_string()],
    };
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create task with Complete status (not WaitingForInput)
    let task = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: "task1".to_string(),
        task_name: "Test Task".to_string(),
        status: TaskStatus::Complete,
        logs: vec!["Task completed".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: chrono::Utc::now().to_rfc3339(),
    };
    store.save_task_execution(&task).await.unwrap();

    // Try to resume task that's not waiting
    let result = resume_task(execution_id, "task1").await;
    assert!(
        result.is_err(),
        "resume_task should fail for non-waiting task"
    );

    // Verify error type
    match result.unwrap_err() {
        s_e_e_core::errors::CoreError::Validation(_) => {
            // Expected error type
        }
        other => panic!("Expected Validation error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_resume_task_multiple_waiting() {
    let execution_id = "test-resume-task-multiple-waiting";

    // Setup test data with multiple waiting tasks
    setup_test_workflow_waiting(execution_id).await.unwrap();

    // Verify initial state
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::WaitingForInput)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task1", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task2", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );

    // Resume only task1
    let result = resume_task(execution_id, "task1").await;
    assert!(result.is_ok(), "resume_task should succeed");

    // Verify task1 status changed
    assert!(
        verify_task_status(execution_id, "task1", TaskStatus::InProgress)
            .await
            .unwrap()
    );

    // Verify task2 still waiting
    assert!(
        verify_task_status(execution_id, "task2", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );

    // Verify workflow status still WaitingForInput (task2 still waiting)
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::WaitingForInput)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_resume_task_last_waiting() {
    let execution_id = "test-resume-task-last-waiting";
    let store = s_e_e_core::get_global_store().unwrap();

    // Setup workflow with WaitingForInput status
    let metadata = WorkflowMetadata {
        id: execution_id.to_string(),
        workflow_name: "Test Workflow".to_string(),
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: None,
        status: WorkflowStatus::WaitingForInput,
        task_ids: vec!["task1".to_string(), "task2".to_string()],
    };
    store.save_workflow_metadata(&metadata).await.unwrap();

    // Create task1 as InProgress (already resumed)
    let task1 = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: "task1".to_string(),
        task_name: "Test Task 1".to_string(),
        status: TaskStatus::InProgress,
        logs: vec!["Task in progress".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task1).await.unwrap();

    // Create task2 as WaitingForInput (last waiting task)
    let task2 = TaskExecution {
        execution_id: execution_id.to_string(),
        task_id: "task2".to_string(),
        task_name: "Test Task 2".to_string(),
        status: TaskStatus::WaitingForInput,
        logs: vec!["Task paused for input".to_string()],
        start_timestamp: chrono::Utc::now().to_rfc3339(),
        end_timestamp: String::new(),
    };
    store.save_task_execution(&task2).await.unwrap();

    // Verify initial state
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::WaitingForInput)
            .await
            .unwrap()
    );
    assert!(
        verify_task_status(execution_id, "task2", TaskStatus::WaitingForInput)
            .await
            .unwrap()
    );

    // Resume the last waiting task
    let result = resume_task(execution_id, "task2").await;
    assert!(result.is_ok(), "resume_task should succeed");

    // Verify task2 status changed
    assert!(
        verify_task_status(execution_id, "task2", TaskStatus::InProgress)
            .await
            .unwrap()
    );

    // Verify workflow status changed to Running (no more waiting tasks)
    assert!(
        verify_workflow_status(execution_id, WorkflowStatus::Running)
            .await
            .unwrap()
    );
}
