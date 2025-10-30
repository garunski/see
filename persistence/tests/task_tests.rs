use chrono::Utc;
use s_e_e_persistence::{Store, TaskExecution, TaskExecutionStatus};

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

fn create_test_task() -> TaskExecution {
    TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskExecutionStatus::Complete,
        output: Some("Task completed successfully".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        user_input: None,
        input_request_id: None,
        prompt_id: None,
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

    store.save_task_execution(task.clone()).await.unwrap();

    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 1);

    let retrieved_task = &tasks[0];
    assert_eq!(retrieved_task.id, "task-1");
    assert_eq!(retrieved_task.workflow_id, "workflow-1");
    assert_eq!(retrieved_task.name, "Test Task");
    assert_eq!(retrieved_task.status, TaskExecutionStatus::Complete);
    assert_eq!(
        retrieved_task.output,
        Some("Task completed successfully".to_string())
    );
}

#[tokio::test]
async fn test_get_tasks_for_workflow_multiple() {
    let store = create_test_store().await;

    let task1 = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Task 1".to_string(),
        status: TaskExecutionStatus::Complete,
        ..Default::default()
    };

    let task2 = TaskExecution {
        id: "task-2".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Task 2".to_string(),
        status: TaskExecutionStatus::Failed,
        ..Default::default()
    };

    let task3 = TaskExecution {
        id: "task-3".to_string(),
        workflow_id: "workflow-2".to_string(),
        name: "Task 3".to_string(),
        status: TaskExecutionStatus::Pending,
        ..Default::default()
    };

    store.save_task_execution(task1).await.unwrap();
    store.save_task_execution(task2).await.unwrap();
    store.save_task_execution(task3).await.unwrap();

    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 2);

    let task_ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert!(task_ids.contains(&"task-1"));
    assert!(task_ids.contains(&"task-2"));
    assert!(!task_ids.contains(&"task-3"));

    let tasks = store.get_tasks_for_workflow("workflow-2").await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "task-3");
}

#[tokio::test]
async fn test_save_task_execution_update() {
    let store = create_test_store().await;
    let mut task = create_test_task();

    store.save_task_execution(task.clone()).await.unwrap();

    task.status = TaskExecutionStatus::Failed;
    task.error = Some("Task failed with error".to_string());
    task.output = None;

    store.save_task_execution(task.clone()).await.unwrap();

    let tasks = store.get_tasks_for_workflow("workflow-1").await.unwrap();
    assert_eq!(tasks.len(), 1);

    let retrieved_task = &tasks[0];
    assert_eq!(retrieved_task.status, TaskExecutionStatus::Failed);
    assert_eq!(
        retrieved_task.error,
        Some("Task failed with error".to_string())
    );
    assert!(retrieved_task.output.is_none());
}

#[tokio::test]
async fn test_task_execution_serialization() {
    let store = create_test_store().await;
    let task = TaskExecution {
        id: "task-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskExecutionStatus::WaitingForInput,
        output: Some("Waiting for user input".to_string()),
        error: None,
        created_at: Utc::now(),
        completed_at: None,
        user_input: None,
        input_request_id: None,
        prompt_id: None,
    };

    store.save_task_execution(task.clone()).await.unwrap();

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
