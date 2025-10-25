use s_e_e_core::engine::custom_engine::{
    convert_workflow_from_json, CustomTask, CustomWorkflow, CustomWorkflowEngine, DependencyGraph,
    TaskFunction, TaskHandler, TaskResult,
};
use s_e_e_core::errors::CoreError;
use s_e_e_core::execution::context::{ExecutionContext, ExecutionContextSafe};
use s_e_e_core::persistence::store::RedbStore;
use s_e_e_core::types::TaskStatus;
use serde_json::{json, Value};
use std::sync::Once;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

static INIT: Once = Once::new();

fn create_test_store() -> Arc<dyn s_e_e_core::AuditStore> {
    INIT.call_once(|| {
        // Initialize any global state if needed
    });

    // Create a unique database path for each test
    let test_id = std::thread::current().id();
    let db_path = format!("/tmp/test_db_{:?}.db", test_id);
    std::fs::remove_file(&db_path).ok(); // Clean up any existing file

    Arc::new(RedbStore::new(db_path.into()).unwrap())
}

/// Mock task handler for testing
struct MockTaskHandler {
    name: String,
    should_fail: bool,
    delay_ms: u64,
}

impl MockTaskHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: false,
            delay_ms: 0,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }
}

#[async_trait::async_trait]
impl TaskHandler for MockTaskHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &CustomTask,
    ) -> Result<TaskResult, CoreError> {
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        context.safe_log(&format!(
            "Mock handler {} executing task: {}",
            self.name, task.name
        ))?;

        if self.should_fail {
            Ok(TaskResult {
                success: false,
                output: Value::Null,
                error: Some(format!("Mock handler {} failed", self.name)),
            })
        } else {
            Ok(TaskResult {
                success: true,
                output: json!({
                    "handler": self.name,
                    "task_id": task.id,
                    "task_name": task.name
                }),
                error: None,
            })
        }
    }
}

#[tokio::test]
async fn test_dependency_graph_no_dependencies() {
    let tasks = vec![
        CustomTask {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            dependencies: vec![],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        },
        CustomTask {
            id: "task2".to_string(),
            name: "Task 2".to_string(),
            dependencies: vec![],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        },
    ];

    let graph = DependencyGraph::new(tasks);
    let completed = std::collections::HashSet::new();
    let running = std::collections::HashSet::new();

    let ready_tasks = graph.get_ready_tasks(&completed, &running);
    assert_eq!(ready_tasks.len(), 2);
    assert!(!graph.has_circular_dependency());
}

#[tokio::test]
async fn test_dependency_graph_with_dependencies() {
    let tasks = vec![
        CustomTask {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            dependencies: vec![],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        },
        CustomTask {
            id: "task2".to_string(),
            name: "Task 2".to_string(),
            dependencies: vec!["task1".to_string()],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        },
    ];

    let graph = DependencyGraph::new(tasks);
    let mut completed = std::collections::HashSet::new();
    let running = std::collections::HashSet::new();

    // Initially only task1 should be ready
    let ready_tasks = graph.get_ready_tasks(&completed, &running);
    assert_eq!(ready_tasks.len(), 1);
    assert_eq!(ready_tasks[0].id, "task1");

    // After task1 is completed, task2 should be ready
    completed.insert("task1".to_string());
    let ready_tasks = graph.get_ready_tasks(&completed, &running);
    assert_eq!(ready_tasks.len(), 1);
    assert_eq!(ready_tasks[0].id, "task2");

    assert!(!graph.has_circular_dependency());
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let tasks = vec![
        CustomTask {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            dependencies: vec!["task2".to_string()],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        },
        CustomTask {
            id: "task2".to_string(),
            name: "Task 2".to_string(),
            dependencies: vec!["task1".to_string()],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        },
    ];

    let graph = DependencyGraph::new(tasks);
    assert!(graph.has_circular_dependency());
}

#[tokio::test]
async fn test_workflow_conversion() {
    let workflow_json = r#"{
        "id": "test_workflow",
        "name": "Test Workflow",
        "tasks": [
            {
                "id": "task1",
                "name": "Task 1",
                "dependencies": [],
                "function": {
                    "type": "custom",
                    "name": "cli_command",
                    "input": {
                        "command": "echo",
                        "args": ["hello"]
                    }
                }
            },
            {
                "id": "task2",
                "name": "Task 2",
                "dependencies": ["task1"],
                "function": {
                    "type": "custom",
                    "name": "cursor_agent",
                    "input": {
                        "prompt": "Generate a greeting"
                    }
                }
            }
        ]
    }"#;

    let custom_workflow = convert_workflow_from_json(workflow_json).unwrap();
    assert_eq!(custom_workflow.name, "Test Workflow");
    assert_eq!(custom_workflow.tasks.len(), 2);

    // Check first task
    match &custom_workflow.tasks[0].function {
        TaskFunction::CliCommand { command, args } => {
            assert_eq!(command, "echo");
            assert_eq!(args, &["hello"]);
        }
        _ => panic!("Expected CliCommand function"),
    }

    // Check second task
    match &custom_workflow.tasks[1].function {
        TaskFunction::CursorAgent { prompt, .. } => {
            assert_eq!(prompt, "Generate a greeting");
        }
        _ => panic!("Expected CursorAgent function"),
    }

    // Check dependencies - now correctly extracted from JSON format
    assert_eq!(custom_workflow.tasks[0].dependencies, Vec::<String>::new());
    assert_eq!(custom_workflow.tasks[1].dependencies, vec!["task1"]);
}

#[tokio::test]
async fn test_custom_engine_execution() {
    let store = create_test_store();
    let mut engine = CustomWorkflowEngine::new(store);

    // Register mock handlers
    engine.register_handler("test".to_string(), Box::new(MockTaskHandler::new("test")));

    let workflow = CustomWorkflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![CustomTask {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            dependencies: vec![],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        }],
    };

    let result = engine.execute_workflow(workflow).await.unwrap();
    assert!(result.success);
    assert_eq!(result.task_count, 1);
    assert_eq!(result.tasks[0].status, TaskStatus::Complete);
}

#[tokio::test]
async fn test_custom_engine_with_dependencies() {
    let store = create_test_store();
    let mut engine = CustomWorkflowEngine::new(store);

    // Register mock handlers
    engine.register_handler("test".to_string(), Box::new(MockTaskHandler::new("test")));

    let workflow = CustomWorkflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ],
    };

    let result = engine.execute_workflow(workflow).await.unwrap();
    assert!(result.success);
    assert_eq!(result.task_count, 2);
    assert_eq!(result.tasks[0].status, TaskStatus::Complete);
    assert_eq!(result.tasks[1].status, TaskStatus::Complete);
}

#[tokio::test]
async fn test_custom_engine_task_failure() {
    let store = create_test_store();
    let mut engine = CustomWorkflowEngine::new(store);

    // Register mock handler that fails
    engine.register_handler(
        "test".to_string(),
        Box::new(MockTaskHandler::new("test").with_failure()),
    );

    let workflow = CustomWorkflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![CustomTask {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            dependencies: vec![],
            function: TaskFunction::Custom {
                name: "test".to_string(),
                input: Value::Null,
            },
            status: TaskStatus::Pending,
        }],
    };

    let result = engine.execute_workflow(workflow).await.unwrap();
    // Engine should continue execution even if a task fails
    assert!(!result.success);
    assert_eq!(result.task_count, 1);
    assert_eq!(result.tasks[0].status, TaskStatus::Failed);
    assert!(!result.errors.is_empty());
}

#[tokio::test]
async fn test_custom_engine_circular_dependency() {
    let store = create_test_store();
    let engine = CustomWorkflowEngine::new(store);

    let workflow = CustomWorkflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec!["task2".to_string()],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ],
    };

    let result = engine.execute_workflow(workflow).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Circular dependency"));
}

#[tokio::test]
async fn test_custom_engine_sequential_execution() {
    let store = create_test_store();
    let mut engine = CustomWorkflowEngine::new(store);

    // Register mock handlers with different delays
    engine.register_handler(
        "fast".to_string(),
        Box::new(MockTaskHandler::new("fast").with_delay(50)),
    );
    engine.register_handler(
        "slow".to_string(),
        Box::new(MockTaskHandler::new("slow").with_delay(100)),
    );

    let workflow = CustomWorkflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "slow".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::Custom {
                    name: "fast".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ],
    };

    let start_time = std::time::Instant::now();
    let result = engine.execute_workflow(workflow).await.unwrap();
    let duration = start_time.elapsed();

    assert!(result.success);
    assert_eq!(result.task_count, 2);

    // Should take at least 150ms (100ms + 50ms) due to sequential execution
    assert!(duration.as_millis() >= 150);
}

#[tokio::test]
async fn test_custom_engine_parallel_ready_tasks() {
    let store = create_test_store();
    let mut engine = CustomWorkflowEngine::new(store);

    // Register mock handlers
    engine.register_handler(
        "test".to_string(),
        Box::new(MockTaskHandler::new("test").with_delay(50)),
    );

    let workflow = CustomWorkflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ],
    };

    let start_time = std::time::Instant::now();
    let result = engine.execute_workflow(workflow).await.unwrap();
    let duration = start_time.elapsed();

    assert!(result.success);
    assert_eq!(result.task_count, 2);

    // Should take at least 100ms (50ms + 50ms) due to sequential execution
    // (even though tasks could run in parallel, current implementation is sequential)
    assert!(duration.as_millis() >= 100);
}
