//! Tests for utility store operations
//!
//! Tests clear_all_data following Single Responsibility Principle.

use s_e_e_persistence::{
    AppSettings, AuditEvent, Prompt, Store, TaskExecution, TaskExecutionStatus, Theme,
    WorkflowDefinition, WorkflowExecution, WorkflowExecutionStatus,
};

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

#[tokio::test]
async fn test_clear_all_data_empty() {
    let store = create_test_store().await;

    // Clear empty database should not error
    let result = store.clear_all_data().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_all_data_with_data() {
    let store = create_test_store().await;

    // Add some test data
    let workflow = WorkflowDefinition {
        id: "workflow-1".to_string(),
        name: "Test Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };

    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        status: WorkflowExecutionStatus::Complete,
        tasks: vec![TaskExecution {
            id: "task-1".to_string(),
            workflow_id: "exec-1".to_string(),
            name: "Test Task".to_string(),
            status: TaskExecutionStatus::Complete,
            ..Default::default()
        }],
        ..Default::default()
    };

    let prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "Test content".to_string(),
        ..Default::default()
    };

    let settings = AppSettings {
        theme: Theme::Dark,
        auto_save: false,
        notifications: true,
        default_workflow: Some("workflow-1".to_string()),
    };

    let audit_event = AuditEvent::success("task-1".to_string(), "Task completed".to_string(), 3);

    // Save all data
    store.save_workflow(&workflow).await.unwrap();
    store.save_workflow_execution(execution).await.unwrap();
    store
        .save_task_execution(TaskExecution {
            id: "task-1".to_string(),
            workflow_id: "exec-1".to_string(),
            name: "Test Task".to_string(),
            status: TaskExecutionStatus::Complete,
            ..Default::default()
        })
        .await
        .unwrap();
    store.save_prompt(&prompt).await.unwrap();
    store.save_settings(&settings).await.unwrap();
    store.log_audit_event(audit_event).await.unwrap();

    // Verify data exists
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 1);

    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 1);

    let tasks = store.get_tasks_for_workflow("exec-1").await.unwrap();
    assert_eq!(tasks.len(), 1);

    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);

    let loaded_settings = store.load_settings().await.unwrap().unwrap();
    assert_eq!(loaded_settings.theme, Theme::Dark);

    // Clear all data
    let result = store.clear_all_data().await;
    assert!(result.is_ok());

    // Verify all data is cleared
    let workflows = store.list_workflows().await.unwrap();
    assert!(workflows.is_empty());

    let executions = store.list_workflow_executions().await.unwrap();
    assert!(executions.is_empty());

    let tasks = store.get_tasks_for_workflow("exec-1").await.unwrap();
    assert!(tasks.is_empty());

    let prompts = store.list_prompts().await.unwrap();
    assert!(prompts.is_empty());

    // Settings should return None after clear
    let loaded_settings = store.load_settings().await.unwrap();
    assert!(loaded_settings.is_none());
}

#[tokio::test]
async fn test_clear_all_data_multiple_times() {
    let store = create_test_store().await;

    // Add data
    let workflow = WorkflowDefinition {
        id: "workflow-1".to_string(),
        name: "Test Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };

    store.save_workflow(&workflow).await.unwrap();

    // Clear first time
    let result = store.clear_all_data().await;
    assert!(result.is_ok());

    // Add data again
    store.save_workflow(&workflow).await.unwrap();

    // Clear second time
    let result = store.clear_all_data().await;
    assert!(result.is_ok());

    // Verify cleared
    let workflows = store.list_workflows().await.unwrap();
    assert!(workflows.is_empty());
}
