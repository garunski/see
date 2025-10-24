use s_e_e_core::execution::context::ExecutionContext;
use s_e_e_core::task_executor::TaskPersistenceHelper;
use s_e_e_core::types::{TaskInfo, TaskStatus};

#[test]
fn test_task_persistence_helper_creation() {
    // Create test context
    let tasks = vec![TaskInfo {
        id: "test_task".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "test_execution".to_string(),
        "test_workflow".to_string(),
    );

    // Create helper - should not panic
    let _helper = TaskPersistenceHelper::new(context);

    // Verify helper can be created (no panic means success)
}

#[tokio::test]
async fn test_save_task_state_without_store() {
    // Test that save_task_state_async handles missing store gracefully
    let tasks = vec![TaskInfo {
        id: "test_task".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None, // No store
        "test_execution".to_string(),
        "test_workflow".to_string(),
    );

    let helper = TaskPersistenceHelper::new(context);

    // Should not panic with no store
    helper.save_task_state_async("test_task", TaskStatus::InProgress);

    // Give async task time to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[test]
fn test_save_all_task_statuses() {
    // Test all three status transitions
    let tasks = vec![TaskInfo {
        id: "test_task".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "test_execution".to_string(),
        "test_workflow".to_string(),
    );

    let helper = TaskPersistenceHelper::new(context);

    // All should complete without panic
    helper.save_task_state_async("test_task", TaskStatus::InProgress);
    helper.save_task_state_async("test_task", TaskStatus::Complete);
    helper.save_task_state_async("test_task", TaskStatus::Failed);
}
