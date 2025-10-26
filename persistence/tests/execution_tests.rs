//! Tests for execution store operations
//! 
//! Tests save/get/list/delete workflow executions, list_workflow_metadata, delete_workflow_metadata_and_tasks, get_workflow_with_tasks following Single Responsibility Principle.

use persistence::{Store, WorkflowExecution, WorkflowStatus, TaskExecution, TaskStatus};
use chrono::Utc;
use std::collections::HashMap;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

fn create_test_execution() -> WorkflowExecution {
    WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: HashMap::new(),
        errors: Vec::new(),
    }
}

#[tokio::test]
async fn test_save_workflow_execution() {
    let store = create_test_store().await;
    let execution = create_test_execution();
    
    let result = store.save_workflow_execution(execution).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_workflow_execution_success() {
    let store = create_test_store().await;
    let execution = create_test_execution();
    
    // Save execution
    store.save_workflow_execution(execution.clone()).await.unwrap();
    
    // Get execution
    let retrieved = store.get_workflow_execution("exec-1").await.unwrap();
    assert!(retrieved.is_some());
    
    let retrieved_execution = retrieved.unwrap();
    assert_eq!(retrieved_execution.id, "exec-1");
    assert_eq!(retrieved_execution.workflow_name, "Test Workflow");
    assert_eq!(retrieved_execution.status, WorkflowStatus::Complete);
    assert_eq!(retrieved_execution.success, Some(true));
}

#[tokio::test]
async fn test_get_workflow_execution_not_found() {
    let store = create_test_store().await;
    
    let retrieved = store.get_workflow_execution("nonexistent").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_list_workflow_executions_empty() {
    let store = create_test_store().await;
    
    let executions = store.list_workflow_executions().await.unwrap();
    assert!(executions.is_empty());
}

#[tokio::test]
async fn test_list_workflow_executions_multiple() {
    let store = create_test_store().await;
    
    // Create multiple executions
    let execution1 = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Workflow 1".to_string(),
        status: WorkflowStatus::Complete,
        ..Default::default()
    };
    
    let execution2 = WorkflowExecution {
        id: "exec-2".to_string(),
        workflow_name: "Workflow 2".to_string(),
        status: WorkflowStatus::Failed,
        ..Default::default()
    };
    
    // Save executions
    store.save_workflow_execution(execution1).await.unwrap();
    store.save_workflow_execution(execution2).await.unwrap();
    
    // List executions
    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 2);
    
    // Check that executions are ordered by ID
    assert_eq!(executions[0].id, "exec-1");
    assert_eq!(executions[1].id, "exec-2");
}

#[tokio::test]
async fn test_delete_workflow_execution() {
    let store = create_test_store().await;
    let execution = create_test_execution();
    
    // Save execution
    store.save_workflow_execution(execution).await.unwrap();
    
    // Verify it exists
    let retrieved = store.get_workflow_execution("exec-1").await.unwrap();
    assert!(retrieved.is_some());
    
    // Delete execution
    let result = store.delete_workflow_execution("exec-1").await;
    assert!(result.is_ok());
    
    // Verify it's gone
    let retrieved = store.get_workflow_execution("exec-1").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_list_workflow_metadata() {
    let store = create_test_store().await;
    
    // Create execution
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowStatus::Running,
        ..Default::default()
    };
    
    store.save_workflow_execution(execution).await.unwrap();
    
    // List metadata
    let metadata = store.list_workflow_metadata().await.unwrap();
    assert_eq!(metadata.len(), 1);
    
    let meta = &metadata[0];
    assert_eq!(meta.id, "exec-1");
    assert_eq!(meta.name, "Test Workflow");
    assert_eq!(meta.status, "running");
}

#[tokio::test]
async fn test_delete_workflow_metadata_and_tasks() {
    let store = create_test_store().await;
    
    // Create execution with tasks
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowStatus::Complete,
        tasks: vec![
            TaskExecution {
                id: "task-1".to_string(),
                workflow_id: "exec-1".to_string(),
                name: "Task 1".to_string(),
                status: TaskStatus::Complete,
                ..Default::default()
            },
            TaskExecution {
                id: "task-2".to_string(),
                workflow_id: "exec-1".to_string(),
                name: "Task 2".to_string(),
                status: TaskStatus::Complete,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    store.save_workflow_execution(execution).await.unwrap();
    
    // Save tasks separately
    let task1 = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "exec-1".to_string(),
        name: "Task 1".to_string(),
        status: TaskStatus::Complete,
        ..Default::default()
    };
    
    let task2 = TaskExecution {
        id: "task-2".to_string(),
        workflow_id: "exec-1".to_string(),
        name: "Task 2".to_string(),
        status: TaskStatus::Complete,
        ..Default::default()
    };
    
    store.save_task_execution(task1).await.unwrap();
    store.save_task_execution(task2).await.unwrap();
    
    // Verify execution and tasks exist
    assert!(store.get_workflow_execution("exec-1").await.unwrap().is_some());
    let tasks = store.get_tasks_for_workflow("exec-1").await.unwrap();
    assert_eq!(tasks.len(), 2);
    
    // Delete execution and tasks
    let result = store.delete_workflow_metadata_and_tasks("exec-1").await;
    assert!(result.is_ok());
    
    // Verify execution and tasks are gone
    assert!(store.get_workflow_execution("exec-1").await.unwrap().is_none());
    let tasks = store.get_tasks_for_workflow("exec-1").await.unwrap();
    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn test_get_workflow_with_tasks() {
    let store = create_test_store().await;
    
    // Create execution without tasks initially
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowStatus::Running,
        tasks: vec![],
        ..Default::default()
    };
    
    store.save_workflow_execution(execution).await.unwrap();
    
    // Save tasks separately
    let task1 = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "exec-1".to_string(),
        name: "Task 1".to_string(),
        status: TaskStatus::Complete,
        ..Default::default()
    };
    
    let task2 = TaskExecution {
        id: "task-2".to_string(),
        workflow_id: "exec-1".to_string(),
        name: "Task 2".to_string(),
        status: TaskStatus::Failed,
        ..Default::default()
    };
    
    store.save_task_execution(task1).await.unwrap();
    store.save_task_execution(task2).await.unwrap();
    
    // Get workflow with tasks
    let execution_with_tasks = store.get_workflow_with_tasks("exec-1").await.unwrap();
    assert_eq!(execution_with_tasks.id, "exec-1");
    assert_eq!(execution_with_tasks.tasks.len(), 2);
    
    // Check task details
    let task_names: Vec<&str> = execution_with_tasks.tasks.iter().map(|t| t.name.as_str()).collect();
    assert!(task_names.contains(&"Task 1"));
    assert!(task_names.contains(&"Task 2"));
}

#[tokio::test]
async fn test_get_workflow_with_tasks_not_found() {
    let store = create_test_store().await;
    
    let result = store.get_workflow_with_tasks("nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}
