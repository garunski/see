//! Integration tests for complete workflow execution flow
//! 
//! Tests multi-table operations, complete workflow execution following Single Responsibility Principle.

use persistence::{Store, WorkflowDefinition, WorkflowExecution, WorkflowStatus, TaskExecution, TaskStatus, UserPrompt, AppSettings, AuditEvent, Theme};
use chrono::Utc;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

#[tokio::test]
async fn test_complete_workflow_execution_flow() {
    let store = create_test_store().await;
    
    // 1. Create and save a workflow
    let workflow = WorkflowDefinition {
        id: "integration-workflow".to_string(),
        name: "Integration Test Workflow".to_string(),
        description: Some("A complete integration test workflow".to_string()),
        content: r#"{"id":"integration","name":"Integration Test","tasks":[{"id":"task1","name":"Task 1"},{"id":"task2","name":"Task 2"}]}"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    store.save_workflow(&workflow).await.unwrap();
    
    // 2. Create and save a workflow execution
    let execution = WorkflowExecution {
        id: "integration-exec-1".to_string(),
        workflow_name: "Integration Test Workflow".to_string(),
        status: WorkflowStatus::Running,
        created_at: Utc::now(),
        completed_at: None,
        success: false,
        tasks: vec![],
        timestamp: Utc::now(),
    };
    
    store.save_workflow_execution(execution.clone()).await.unwrap();
    
    // 3. Create and save tasks for the execution
    let task1 = TaskExecution {
        id: "integration-task-1".to_string(),
        workflow_id: "integration-exec-1".to_string(),
        name: "Task 1".to_string(),
        status: TaskStatus::Complete,
        output: Some("Task 1 completed successfully".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    
    let task2 = TaskExecution {
        id: "integration-task-2".to_string(),
        workflow_id: "integration-exec-1".to_string(),
        name: "Task 2".to_string(),
        status: TaskStatus::Failed,
        output: None,
        error: Some("Task 2 failed with error".to_string()),
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    
    store.save_task_execution(task1.clone()).await.unwrap();
    store.save_task_execution(task2.clone()).await.unwrap();
    
    // 4. Log audit events for tasks
    let audit1 = AuditEvent::success(
        "integration-task-1".to_string(),
        "Task 1 completed successfully".to_string(),
        5
    );
    
    let audit2 = AuditEvent::failure(
        "integration-task-2".to_string(),
        "Task 2 failed with error".to_string(),
        0
    );
    
    store.log_audit_event(audit1).await.unwrap();
    store.log_audit_event(audit2).await.unwrap();
    
    // 5. Update execution status to complete
    let mut updated_execution = execution;
    updated_execution.status = WorkflowStatus::Complete;
    updated_execution.completed_at = Some(Utc::now());
    updated_execution.success = false; // One task failed
    
    store.save_workflow_execution(updated_execution).await.unwrap();
    
    // 6. Verify complete workflow execution data
    let retrieved_execution = store.get_workflow_with_tasks("integration-exec-1").await.unwrap();
    assert_eq!(retrieved_execution.id, "integration-exec-1");
    assert_eq!(retrieved_execution.status, WorkflowStatus::Complete);
    assert_eq!(retrieved_execution.tasks.len(), 2);
    
    // Verify task details
    let task1_retrieved = retrieved_execution.tasks.iter().find(|t| t.id == "integration-task-1").unwrap();
    assert_eq!(task1_retrieved.status, TaskStatus::Complete);
    assert_eq!(task1_retrieved.output, Some("Task 1 completed successfully".to_string()));
    
    let task2_retrieved = retrieved_execution.tasks.iter().find(|t| t.id == "integration-task-2").unwrap();
    assert_eq!(task2_retrieved.status, TaskStatus::Failed);
    assert_eq!(task2_retrieved.error, Some("Task 2 failed with error".to_string()));
    
    // 7. Verify workflow metadata
    let metadata = store.list_workflow_metadata().await.unwrap();
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].id, "integration-exec-1");
    assert_eq!(metadata[0].name, "Integration Test Workflow");
    assert_eq!(metadata[0].status, "complete");
}

#[tokio::test]
async fn test_multi_table_operations() {
    let store = create_test_store().await;
    
    // Create data across all tables
    let workflow = WorkflowDefinition {
        id: "multi-table-workflow".to_string(),
        name: "Multi Table Workflow".to_string(),
        content: r#"{"id":"multi","name":"Multi","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    let execution = WorkflowExecution {
        id: "multi-exec-1".to_string(),
        workflow_name: "Multi Table Workflow".to_string(),
        status: WorkflowStatus::Running,
        ..Default::default()
    };
    
    let task = TaskExecution {
        id: "multi-task-1".to_string(),
        workflow_id: "multi-exec-1".to_string(),
        name: "Multi Task".to_string(),
        status: TaskStatus::Complete,
        ..Default::default()
    };
    
    let prompt = UserPrompt {
        id: "multi-prompt-1".to_string(),
        name: "Multi Prompt".to_string(),
        content: "Multi table test prompt".to_string(),
        ..Default::default()
    };
    
    let settings = AppSettings {
        theme: Theme::Dark,
        auto_save: true,
        notifications: false,
        default_workflow: Some("multi-table-workflow".to_string()),
    };
    
    let audit = AuditEvent::success(
        "multi-task-1".to_string(),
        "Multi table test completed".to_string(),
        3
    );
    
    // Save all data
    store.save_workflow(&workflow).await.unwrap();
    store.save_workflow_execution(execution).await.unwrap();
    store.save_task_execution(task).await.unwrap();
    store.save_prompt(&prompt).await.unwrap();
    store.save_settings(&settings).await.unwrap();
    store.log_audit_event(audit).await.unwrap();
    
    // Verify all data exists
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 1);
    
    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 1);
    
    let tasks = store.get_tasks_for_workflow("multi-exec-1").await.unwrap();
    assert_eq!(tasks.len(), 1);
    
    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);
    
    let loaded_settings = store.load_settings().await.unwrap().unwrap();
    assert_eq!(loaded_settings.theme, Theme::Dark);
    assert_eq!(loaded_settings.default_workflow, Some("multi-table-workflow".to_string()));
}

#[tokio::test]
async fn test_workflow_execution_with_prompts_and_settings() {
    let store = create_test_store().await;
    
    // Set up settings
    let settings = AppSettings {
        theme: Theme::Light,
        auto_save: true,
        notifications: true,
        default_workflow: None,
    };
    
    store.save_settings(&settings).await.unwrap();
    
    // Create prompts
    let prompt1 = UserPrompt {
        id: "exec-prompt-1".to_string(),
        name: "Execution Prompt 1".to_string(),
        content: "First prompt for execution".to_string(),
        ..Default::default()
    };
    
    let prompt2 = UserPrompt {
        id: "exec-prompt-2".to_string(),
        name: "Execution Prompt 2".to_string(),
        content: "Second prompt for execution".to_string(),
        ..Default::default()
    };
    
    store.save_prompt(&prompt1).await.unwrap();
    store.save_prompt(&prompt2).await.unwrap();
    
    // Create workflow
    let workflow = WorkflowDefinition {
        id: "exec-workflow".to_string(),
        name: "Execution Workflow".to_string(),
        content: r#"{"id":"exec","name":"Execution","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    store.save_workflow(&workflow).await.unwrap();
    
    // Create execution
    let execution = WorkflowExecution {
        id: "exec-exec-1".to_string(),
        workflow_name: "Execution Workflow".to_string(),
        status: WorkflowStatus::Complete,
        success: true,
        ..Default::default()
    };
    
    store.save_workflow_execution(execution).await.unwrap();
    
    // Verify complete state
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 1);
    
    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 1);
    assert!(executions[0].success);
    
    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 2);
    
    let loaded_settings = store.load_settings().await.unwrap().unwrap();
    assert_eq!(loaded_settings.theme, Theme::Light);
    assert!(loaded_settings.auto_save);
    assert!(loaded_settings.notifications);
}

#[tokio::test]
async fn test_data_cleanup_and_recreation() {
    let store = create_test_store().await;
    
    // Create initial data
    let workflow = WorkflowDefinition {
        id: "cleanup-workflow".to_string(),
        name: "Cleanup Workflow".to_string(),
        content: r#"{"id":"cleanup","name":"Cleanup","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    let execution = WorkflowExecution {
        id: "cleanup-exec-1".to_string(),
        workflow_name: "Cleanup Workflow".to_string(),
        status: WorkflowStatus::Complete,
        ..Default::default()
    };
    
    store.save_workflow(&workflow).await.unwrap();
    store.save_workflow_execution(execution).await.unwrap();
    
    // Verify data exists
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 1);
    
    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 1);
    
    // Clear all data
    store.clear_all_data().await.unwrap();
    
    // Verify data is cleared
    let workflows = store.list_workflows().await.unwrap();
    assert!(workflows.is_empty());
    
    let executions = store.list_workflow_executions().await.unwrap();
    assert!(executions.is_empty());
    
    // Recreate data
    let new_workflow = WorkflowDefinition {
        id: "recreated-workflow".to_string(),
        name: "Recreated Workflow".to_string(),
        content: r#"{"id":"recreated","name":"Recreated","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    let new_execution = WorkflowExecution {
        id: "recreated-exec-1".to_string(),
        workflow_name: "Recreated Workflow".to_string(),
        status: WorkflowStatus::Running,
        ..Default::default()
    };
    
    store.save_workflow(&new_workflow).await.unwrap();
    store.save_workflow_execution(new_execution).await.unwrap();
    
    // Verify recreated data
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].id, "recreated-workflow");
    
    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0].id, "recreated-exec-1");
}
