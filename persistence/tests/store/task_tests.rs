//! Tests for task store operations
//! 
//! Tests save_task_execution, get_tasks_for_workflow following Single Responsibility Principle.

use persistence::{Store, TaskExecution, TaskStatus};
use chrono::Utc;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

fn create_test_task() -> TaskExecution {
    TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Complete,
        output: Some("Task completed successfully".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
    }
}

#[tokio::test]
async fn test_save_task_execution() {
    let store = create_test_store().await;
    let task = create_test_task();
    
    let result = store.save_task_execution(task).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_tasks_for_workflow_empty() {
    let store = create_test_store().await;
    
    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert!(tasks.is_empty());
}

#[tokio::test]
async fn test_get_tasks_for_workflow_single() {
    let store = create_test_store().await;
    let task = create_test_task();
    
    // Save task
    store.save_task_execution(task.clone()).await.unwrap();
    
    // Get tasks for workflow
    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 1);
    
    let retrieved_task = &tasks[0];
    assert_eq!(retrieved_task.id, "task-1");
    assert_eq!(retrieved_task.workflow_id, "workflow-1");
    assert_eq!(retrieved_task.name, "Test Task");
    assert_eq!(retrieved_task.status, TaskStatus::Complete);
    assert_eq!(retrieved_task.output, Some("Task completed successfully".to_string()));
}

#[tokio::test]
async fn test_get_tasks_for_workflow_multiple() {
    let store = create_test_store().await;
    
    // Create multiple tasks for same workflow
    let task1 = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Task 1".to_string(),
        status: TaskStatus::Complete,
        ..Default::default()
    };
    
    let task2 = TaskExecution {
        id: "task-2".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Task 2".to_string(),
        status: TaskStatus::Failed,
        ..Default::default()
    };
    
    let task3 = TaskExecution {
        id: "task-3".to_string(),
        workflow_id: "workflow-2".to_string(), // Different workflow
        name: "Task 3".to_string(),
        status: TaskStatus::Pending,
        ..Default::default()
    };
    
    // Save tasks
    store.save_task_execution(task1).await.unwrap();
    store.save_task_execution(task2).await.unwrap();
    store.save_task_execution(task3).await.unwrap();
    
    // Get tasks for workflow-1
    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 2);
    
    // Check task IDs
    let task_ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert!(task_ids.contains(&"task-1"));
    assert!(task_ids.contains(&"task-2"));
    assert!(!task_ids.contains(&"task-3"));
    
    // Get tasks for workflow-2
    let tasks = store.get_tasks_for_workflow("workflow-2").await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "task-3");
}

#[tokio::test]
async fn test_save_task_execution_update() {
    let store = create_test_store().await;
    let mut task = create_test_task();
    
    // Save initial task
    store.save_task_execution(task.clone()).await.unwrap();
    
    // Update task
    task.status = TaskStatus::Failed;
    task.error = Some("Task failed with error".to_string());
    task.output = None;
    
    // Save updated task
    store.save_task_execution(task.clone()).await.unwrap();
    
    // Verify update
    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 1);
    
    let retrieved_task = &tasks[0];
    assert_eq!(retrieved_task.status, TaskStatus::Failed);
    assert_eq!(retrieved_task.error, Some("Task failed with error".to_string()));
    assert!(retrieved_task.output.is_none());
}

#[tokio::test]
async fn test_task_execution_serialization() {
    let store = create_test_store().await;
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::WaitingForInput,
        output: Some("Waiting for user input".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: None,
    };
    
    // Save task
    store.save_task_execution(task.clone()).await.unwrap();
    
    // Retrieve and verify
    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 1);
    
    let retrieved_task = &tasks[0];
    assert_eq!(retrieved_task.id, task.id);
    assert_eq!(retrieved_task.workflow_id, task.workflow_id);
    assert_eq!(retrieved_task.name, task.name);
    assert_eq!(retrieved_task.status, task.status);
    assert_eq!(retrieved_task.output, task.output);
    assert_eq!(retrieved_task.error, task.error);
}

#[tokio::test]
async fn test_save_task_with_input() {
    let store = create_test_store().await;
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::WaitingForInput,
        output: None,
        error: None,
        created_at: Utc::now(),
        completed_at: None,
        user_input: Some("user-input".to_string()),
        input_request_id: Some("request-123".to_string()),
        prompt_id: None,
    };
    
    let result = store.save_task_with_input(task.clone()).await;
    assert!(result.is_ok());
    
    // Verify task was saved
    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].user_input, Some("user-input".to_string()));
}

#[tokio::test]
async fn test_get_tasks_waiting_for_input() {
    let store = create_test_store().await;
    
    // Create tasks with different statuses
    let task1 = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Waiting Task".to_string(),
        status: TaskStatus::WaitingForInput,
        ..Default::default()
    };
    
    let task2 = TaskExecution {
        id: "task-2".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Complete Task".to_string(),
        status: TaskStatus::Complete,
        completed_at: Some(Utc::now()),
        ..Default::default()
    };
    
    let task3 = TaskExecution {
        id: "task-3".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Another Waiting Task".to_string(),
        status: TaskStatus::WaitingForInput,
        ..Default::default()
    };
    
    // Save all tasks
    store.save_task_execution(task1).await.unwrap();
    store.save_task_execution(task2).await.unwrap();
    store.save_task_execution(task3).await.unwrap();
    
    // Get tasks waiting for input
    let waiting_tasks = store.get_tasks_waiting_for_input().await.unwrap();
    assert_eq!(waiting_tasks.len(), 2);
    
    // Verify IDs
    let task_ids: Vec<&str> = waiting_tasks.iter().map(|t| t.id.as_str()).collect();
    assert!(task_ids.contains(&"task-1"));
    assert!(task_ids.contains(&"task-3"));
    assert!(!task_ids.contains(&"task-2")); // Complete task
}

#[tokio::test]
async fn test_get_tasks_waiting_for_input_in_workflow() {
    let store = create_test_store().await;
    
    // Create tasks in different workflows
    let task1 = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Waiting Task 1".to_string(),
        status: TaskStatus::WaitingForInput,
        ..Default::default()
    };
    
    let task2 = TaskExecution {
        id: "task-2".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Waiting Task 2".to_string(),
        status: TaskStatus::WaitingForInput,
        ..Default::default()
    };
    
    let task3 = TaskExecution {
        id: "task-3".to_string(),
        workflow_id: "workflow-2".to_string(),
        name: "Waiting Task 3".to_string(),
        status: TaskStatus::WaitingForInput,
        ..Default::default()
    };
    
    store.save_task_execution(task1).await.unwrap();
    store.save_task_execution(task2).await.unwrap();
    store.save_task_execution(task3).await.unwrap();
    
    // Get waiting tasks for workflow-1
    let waiting = store.get_tasks_waiting_for_input_in_workflow("workflow-1").await.unwrap();
    assert_eq!(waiting.len(), 2);
    
    // Get waiting tasks for workflow-2
    let waiting = store.get_tasks_waiting_for_input_in_workflow("workflow-2").await.unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].id, "task-3");
}

#[tokio::test]
async fn test_get_task_with_input_request() {
    let store = create_test_store().await;
    
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::WaitingForInput,
        input_request_id: Some("request-123".to_string()),
        ..Default::default()
    };
    
    // Save task
    store.save_task_execution(task).await.unwrap();
    
    // Get task with input request
    let retrieved = store.get_task_with_input_request("task-1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "task-1");
    
    // Try to get non-existent task
    let not_found = store.get_task_with_input_request("non-existent").await.unwrap();
    assert!(not_found.is_none());
}
