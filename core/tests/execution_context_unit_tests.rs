use s_e_e_core::errors::CoreError;
use s_e_e_core::execution::context::ExecutionContext;
use s_e_e_core::types::{TaskInfo, TaskStatus};
use std::sync::{Arc, Mutex};

fn create_test_context() -> Arc<Mutex<ExecutionContext>> {
    let tasks = vec![
        TaskInfo {
            id: "task1".to_string(),
            name: "Test Task 1".to_string(),
            status: TaskStatus::Pending,
        },
        TaskInfo {
            id: "task2".to_string(),
            name: "Test Task 2".to_string(),
            status: TaskStatus::InProgress,
        },
    ];

    ExecutionContext::new(
        tasks,
        None,
        None,
        "test_execution_id".to_string(),
        "test_workflow".to_string(),
    )
}

#[test]
fn test_pause_for_input_success() {
    let context = create_test_context();
    let mut ctx = context.lock().unwrap();

    // Start the task first
    ctx.start_task("task1");

    // Pause the task
    let result = ctx.pause_for_input("task1", "Continue with operation?");
    assert!(result.is_ok(), "pause_for_input should succeed");

    // Verify task status changed
    let tasks = ctx.get_tasks();
    let task = tasks.iter().find(|t| t.id == "task1").unwrap();
    assert_eq!(
        task.status,
        TaskStatus::WaitingForInput,
        "Task should be waiting for input"
    );
}

#[test]
fn test_pause_for_input_nonexistent_task() {
    let context = create_test_context();
    let mut ctx = context.lock().unwrap();

    // Try to pause non-existent task
    let result = ctx.pause_for_input("nonexistent_task", "Continue?");
    assert!(
        result.is_err(),
        "pause_for_input should fail for nonexistent task"
    );

    // Verify error message
    match result {
        Err(CoreError::Validation(msg)) => {
            assert!(
                msg.contains("not found"),
                "Error should mention task not found"
            );
        }
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_resume_task_success() {
    let context = create_test_context();

    // First pause the task
    {
        let mut ctx = context.lock().unwrap();
        ctx.start_task("task1");
        ctx.pause_for_input("task1", "Continue?").unwrap();
    }

    // Now resume it
    {
        let mut ctx = context.lock().unwrap();
        let result = ctx.resume_task("task1");
        assert!(result.is_ok(), "resume_task should succeed");

        // Verify task status changed back
        let tasks = ctx.get_tasks();
        let task = tasks.iter().find(|t| t.id == "task1").unwrap();
        assert_eq!(
            task.status,
            TaskStatus::InProgress,
            "Task should be in progress after resume"
        );
    }
}

#[test]
fn test_resume_task_nonexistent_task() {
    let context = create_test_context();
    let mut ctx = context.lock().unwrap();

    // Try to resume non-existent task
    let result = ctx.resume_task("nonexistent_task");
    assert!(
        result.is_err(),
        "resume_task should fail for nonexistent task"
    );

    match result {
        Err(CoreError::Validation(msg)) => {
            assert!(
                msg.contains("not found"),
                "Error should mention task not found"
            );
        }
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_resume_task_wrong_status() {
    let context = create_test_context();
    let mut ctx = context.lock().unwrap();

    // Task is in Pending status, not WaitingForInput
    let result = ctx.resume_task("task1");
    assert!(
        result.is_err(),
        "resume_task should fail for task not waiting for input"
    );

    match result {
        Err(CoreError::Validation(msg)) => {
            assert!(
                msg.contains("not waiting for input"),
                "Error should mention wrong status"
            );
        }
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_has_waiting_tasks() {
    let context = create_test_context();

    // Initially no waiting tasks
    {
        let ctx = context.lock().unwrap();
        assert!(
            !ctx.has_waiting_tasks(),
            "Should have no waiting tasks initially"
        );
    }

    // Pause a task
    {
        let mut ctx = context.lock().unwrap();
        ctx.start_task("task1");
        ctx.pause_for_input("task1", "Continue?").unwrap();
    }

    // Now should have waiting tasks
    {
        let ctx = context.lock().unwrap();
        assert!(
            ctx.has_waiting_tasks(),
            "Should have waiting tasks after pause"
        );
    }
}

#[test]
fn test_get_waiting_tasks() {
    let context = create_test_context();

    // Initially no waiting tasks
    {
        let ctx = context.lock().unwrap();
        let waiting = ctx.get_waiting_tasks();
        assert_eq!(waiting.len(), 0, "Should have no waiting tasks initially");
    }

    // Pause a task
    {
        let mut ctx = context.lock().unwrap();
        ctx.start_task("task1");
        ctx.pause_for_input("task1", "Continue?").unwrap();
    }

    // Now should have one waiting task
    {
        let ctx = context.lock().unwrap();
        let waiting = ctx.get_waiting_tasks();
        assert_eq!(waiting.len(), 1, "Should have one waiting task");
        assert_eq!(waiting[0].id, "task1", "Waiting task should be task1");
    }
}

#[test]
fn test_pause_resume_logs() {
    let context = create_test_context();
    let mut ctx = context.lock().unwrap();

    // Start and pause
    ctx.start_task("task1");
    ctx.pause_for_input("task1", "Test prompt").unwrap();

    // Check logs contain pause message
    let logs = ctx.get_output_logs();
    let has_pause_log = logs.iter().any(|log| log.contains("paused for user input"));
    assert!(has_pause_log, "Logs should contain pause message");

    // Resume
    ctx.resume_task("task1").unwrap();

    // Check logs contain resume message
    let logs = ctx.get_output_logs();
    let has_resume_log = logs
        .iter()
        .any(|log| log.contains("resumed from user input pause"));
    assert!(has_resume_log, "Logs should contain resume message");
}

#[test]
fn test_multiple_tasks_pause_resume() {
    let context = create_test_context();

    // Pause both tasks
    {
        let mut ctx = context.lock().unwrap();
        ctx.start_task("task1");
        ctx.pause_for_input("task1", "Continue task1?").unwrap();
        ctx.start_task("task2");
        ctx.pause_for_input("task2", "Continue task2?").unwrap();
    }

    // Both should be waiting
    {
        let ctx = context.lock().unwrap();
        assert!(ctx.has_waiting_tasks(), "Should have waiting tasks");
        let waiting = ctx.get_waiting_tasks();
        assert_eq!(waiting.len(), 2, "Should have two waiting tasks");
    }

    // Resume first task
    {
        let mut ctx = context.lock().unwrap();
        ctx.resume_task("task1").unwrap();
    }

    // Should still have one waiting
    {
        let ctx = context.lock().unwrap();
        assert!(ctx.has_waiting_tasks(), "Should still have waiting tasks");
        let waiting = ctx.get_waiting_tasks();
        assert_eq!(waiting.len(), 1, "Should have one waiting task");
        assert_eq!(waiting[0].id, "task2", "Waiting task should be task2");
    }
}
