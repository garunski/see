use simple_workflow_app::execution_context::ExecutionContext;
use simple_workflow_app::TaskInfo;
use std::sync::{Arc, Mutex};

#[test]
fn test_execution_context_log_parsing() {
    let tasks = vec![TaskInfo {
        id: "test_task".to_string(),
        name: "Test".to_string(),
        status: "pending".to_string(),
    }];

    let context = ExecutionContext::new(tasks, None);
    let mut ctx = context.lock().unwrap();

    ctx.log("[TASK_START:test_task]");
    ctx.log("Task output");
    ctx.log("[TASK_END:test_task]");

    let logs = ctx.get_per_task_logs();
    assert_eq!(
        logs.get("test_task").unwrap(),
        &vec!["Task output".to_string()]
    );
}

#[test]
fn test_execution_context_filters_markers() {
    let context = ExecutionContext::new(vec![], None);
    let mut ctx = context.lock().unwrap();

    ctx.log("[TASK_START:test]");
    ctx.log("Real output");
    ctx.log("[TASK_END:test]");

    let output_logs = ctx.get_output_logs();
    assert_eq!(output_logs, vec!["Real output".to_string()]);
}

#[test]
fn test_execution_context_task_status_updates() {
    let tasks = vec![TaskInfo {
        id: "test_task".to_string(),
        name: "Test".to_string(),
        status: "pending".to_string(),
    }];

    let context = ExecutionContext::new(tasks, None);
    let mut ctx = context.lock().unwrap();

    ctx.log("[TASK_START:test_task]");

    let tasks = ctx.get_tasks();
    assert_eq!(tasks[0].status, "in-progress");
}

#[test]
fn test_execution_context_callback_invocation() {
    let called = Arc::new(Mutex::new(Vec::new()));
    let called_clone = called.clone();

    let callback = Arc::new(move |msg: String| {
        called_clone.lock().unwrap().push(msg);
    });

    let context = ExecutionContext::new(vec![], Some(callback));
    let mut ctx = context.lock().unwrap();

    ctx.log("Test message");

    drop(ctx);
    assert_eq!(
        called.lock().unwrap().as_slice(),
        &["Test message".to_string()]
    );
}

#[test]
fn test_execution_context_multiple_tasks() {
    let tasks = vec![
        TaskInfo {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            status: "pending".to_string(),
        },
        TaskInfo {
            id: "task2".to_string(),
            name: "Task 2".to_string(),
            status: "pending".to_string(),
        },
    ];

    let context = ExecutionContext::new(tasks, None);
    let mut ctx = context.lock().unwrap();

    // First task
    ctx.log("[TASK_START:task1]");
    ctx.log("Task 1 output");
    ctx.log("[TASK_END:task1]");

    // Second task
    ctx.log("[TASK_START:task2]");
    ctx.log("Task 2 output");
    ctx.log("[TASK_END:task2]");

    let logs = ctx.get_per_task_logs();
    assert_eq!(
        logs.get("task1").unwrap(),
        &vec!["Task 1 output".to_string()]
    );
    assert_eq!(
        logs.get("task2").unwrap(),
        &vec!["Task 2 output".to_string()]
    );

    // Output logs should not contain markers
    let output_logs = ctx.get_output_logs();
    assert_eq!(
        output_logs,
        vec!["Task 1 output".to_string(), "Task 2 output".to_string()]
    );
}

#[test]
fn test_execution_context_extract_data() {
    let tasks = vec![TaskInfo {
        id: "test_task".to_string(),
        name: "Test".to_string(),
        status: "pending".to_string(),
    }];

    let context = ExecutionContext::new(tasks, None);
    {
        let mut ctx = context.lock().unwrap();

        ctx.log("[TASK_START:test_task]");
        ctx.log("Task output");
        ctx.log("[TASK_END:test_task]");

        // Update task status
        ctx.update_task_status("test_task", "complete");
    }

    // Extract data after dropping the lock
    let (output_logs, per_task_logs, tasks) = Arc::try_unwrap(context)
        .map_err(|_| "Failed to unwrap Arc")
        .unwrap()
        .into_inner()
        .map_err(|_| "Failed to unwrap Mutex")
        .unwrap()
        .extract_data();

    assert_eq!(output_logs, vec!["Task output".to_string()]);
    assert_eq!(
        per_task_logs.get("test_task").unwrap(),
        &vec!["Task output".to_string()]
    );
    assert_eq!(tasks[0].status, "complete");
}
