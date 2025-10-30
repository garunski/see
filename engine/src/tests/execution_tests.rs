use crate::*;

#[tokio::test]
async fn test_per_task_logs_captured() {
    let workflow = EngineWorkflow {
        id: "test".to_string(),
        name: "Test Logs".to_string(),
        tasks: vec![EngineTask {
            id: "task1".to_string(),
            name: "Echo Task".to_string(),
            function: TaskFunction::CliCommand {
                command: "echo".to_string(),
                args: vec!["Hello World".to_string()],
            },
            next_tasks: vec![],
            status: TaskStatus::Pending,
            is_root: true,
        }],
    };

    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();

    assert!(result.per_task_logs.contains_key("task1"));
    let logs = &result.per_task_logs["task1"];
    assert!(!logs.is_empty());
    assert!(logs.iter().any(|log| log.contains("Hello World")));
}

#[tokio::test]
async fn test_parallel_tasks_separate_logs() {
    let workflow = EngineWorkflow {
        id: "parallel".to_string(),
        name: "Parallel Test".to_string(),
        tasks: vec![
            EngineTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Task1Output".to_string()],
                },
                next_tasks: vec![],
                status: TaskStatus::Pending,
                is_root: true,
            },
            EngineTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Task2Output".to_string()],
                },
                next_tasks: vec![],
                status: TaskStatus::Pending,
                is_root: true,
            },
        ],
    };

    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();

    assert_eq!(result.per_task_logs.len(), 2);
    assert!(result.per_task_logs["task1"]
        .iter()
        .any(|log| log.contains("Task1Output")));
    assert!(result.per_task_logs["task2"]
        .iter()
        .any(|log| log.contains("Task2Output")));
}

#[tokio::test]
async fn test_error_capture_in_logs() {
    let workflow = EngineWorkflow {
        id: "error".to_string(),
        name: "Error Test".to_string(),
        tasks: vec![EngineTask {
            id: "failing_task".to_string(),
            name: "Failing Task".to_string(),
            function: TaskFunction::CliCommand {
                command: "nonexistent_command_xyz".to_string(),
                args: vec![],
            },
            next_tasks: vec![],
            status: TaskStatus::Pending,
            is_root: true,
        }],
    };

    let engine = WorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await.unwrap();

    assert!(result.per_task_logs.contains_key("failing_task"));
    let logs = &result.per_task_logs["failing_task"];
    assert!(logs.iter().any(|log| log.contains("Error:")));
}
