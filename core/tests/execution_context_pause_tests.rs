use s_e_e_core::execution::context::ExecutionContext;
use s_e_e_core::types::{TaskInfo, TaskStatus};

#[test]
fn integration_test_full_pause_resume_workflow() {
    // Create context with multiple tasks
    let tasks = vec![
        TaskInfo {
            id: "task1".to_string(),
            name: "First Task".to_string(),
            status: TaskStatus::Pending,
        },
        TaskInfo {
            id: "task2".to_string(),
            name: "Second Task".to_string(),
            status: TaskStatus::Pending,
        },
        TaskInfo {
            id: "task3".to_string(),
            name: "Third Task".to_string(),
            status: TaskStatus::Pending,
        },
    ];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "integration_test_execution".to_string(),
        "integration_test_workflow".to_string(),
    );

    // Simulate workflow: start task1, pause, resume, complete
    {
        let mut ctx = context.lock().unwrap();

        // Start task 1
        ctx.start_task("task1");

        // Pause task 1
        ctx.pause_for_input("task1", "Proceed with task1?").unwrap();
        assert!(ctx.has_waiting_tasks());

        // Resume task 1
        ctx.resume_task("task1").unwrap();
        assert!(!ctx.has_waiting_tasks());

        // Complete task 1
        ctx.update_task_status("task1", TaskStatus::Complete);
        ctx.end_task("task1");
    }

    // Verify final state
    {
        let ctx = context.lock().unwrap();
        let tasks = ctx.get_tasks();
        let task1 = tasks.iter().find(|t| t.id == "task1").unwrap();
        assert_eq!(task1.status, TaskStatus::Complete);
        assert!(!ctx.has_waiting_tasks());
    }
}

#[test]
fn integration_test_error_recovery() {
    let tasks = vec![TaskInfo {
        id: "task1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Pending,
    }];

    let context = ExecutionContext::new(
        tasks,
        None,
        None,
        "error_test_execution".to_string(),
        "error_test_workflow".to_string(),
    );

    let mut ctx = context.lock().unwrap();

    // Try to pause without starting
    let result = ctx.pause_for_input("task1", "Test");
    assert!(
        result.is_ok(),
        "Should be able to pause even if not started"
    );

    // Try to resume twice (second should fail)
    ctx.resume_task("task1").unwrap();
    let result = ctx.resume_task("task1");
    assert!(result.is_err(), "Second resume should fail");

    // Try to pause non-existent task
    let result = ctx.pause_for_input("nonexistent", "Test");
    assert!(result.is_err(), "Should fail for non-existent task");
}
