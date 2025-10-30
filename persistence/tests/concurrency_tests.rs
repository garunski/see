use s_e_e_persistence::{
    Store, TaskExecution, TaskExecutionStatus, WorkflowDefinition, WorkflowExecution,
    WorkflowExecutionStatus,
};
use std::sync::Arc;
use tokio::task;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

#[tokio::test]
async fn test_concurrent_readers() {
    let store = Arc::new(create_test_store().await);

    let workflow = WorkflowDefinition {
        id: "concurrent-test".to_string(),
        name: "Concurrent Test Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };

    store.save_workflow(&workflow).await.unwrap();

    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = task::spawn(async move {
            for _j in 0..5 {
                let workflows = store_clone.list_workflows().await.unwrap();
                assert_eq!(workflows.len(), 1);
                assert_eq!(workflows[0].id, "concurrent-test");

                let retrieved = store_clone.get_workflow("concurrent-test").await.unwrap();
                assert!(retrieved.is_some());
                assert_eq!(retrieved.unwrap().name, "Concurrent Test Workflow");

                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
            format!("reader_{}", i)
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.starts_with("reader_"));
    }
}

#[tokio::test]
async fn test_concurrent_writers() {
    let store = Arc::new(create_test_store().await);

    let mut handles = vec![];

    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = task::spawn(async move {
            for j in 0..3 {
                let workflow = WorkflowDefinition {
                    id: format!("workflow_{}_{}", i, j),
                    name: format!("Workflow {}-{}", i, j),
                    content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
                    ..Default::default()
                };

                let result = store_clone.save_workflow(&workflow).await;
                assert!(result.is_ok());

                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
            i
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result < 5);
    }

    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 15);
}

#[tokio::test]
async fn test_read_during_write() {
    let store = Arc::new(create_test_store().await);

    let initial_workflow = WorkflowDefinition {
        id: "initial".to_string(),
        name: "Initial Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };

    store.save_workflow(&initial_workflow).await.unwrap();

    let writer_store = Arc::clone(&store);
    let writer_handle = task::spawn(async move {
        for i in 0..10 {
            let workflow = WorkflowDefinition {
                id: format!("writer_{}", i),
                name: format!("Writer Workflow {}", i),
                content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
                ..Default::default()
            };

            writer_store.save_workflow(&workflow).await.unwrap();

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });

    let reader_store = Arc::clone(&store);
    let reader_handle = task::spawn(async move {
        let mut read_count = 0;
        while read_count < 50 {
            let workflows = reader_store.list_workflows().await.unwrap();
            assert!(!workflows.is_empty());

            let retrieved = reader_store.get_workflow("initial").await.unwrap();
            assert!(retrieved.is_some());

            read_count += 1;

            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
        read_count
    });

    writer_handle.await.unwrap();
    let reader_result = reader_handle.await.unwrap();

    assert_eq!(reader_result, 50);

    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 11);
}

#[tokio::test]
async fn test_concurrent_execution_operations() {
    let store = Arc::new(create_test_store().await);

    let mut handles = vec![];

    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = task::spawn(async move {
            let execution = WorkflowExecution {
                id: format!("exec_{}", i),
                workflow_name: format!("Workflow {}", i),
                status: WorkflowExecutionStatus::Running,
                tasks: vec![],
                ..Default::default()
            };

            store_clone
                .save_workflow_execution(execution)
                .await
                .unwrap();

            for j in 0..3 {
                let task = TaskExecution {
                    id: format!("task_{}_{}", i, j),
                    workflow_id: format!("exec_{}", i),
                    name: format!("Task {}-{}", i, j),
                    status: TaskExecutionStatus::Complete,
                    ..Default::default()
                };

                store_clone.save_task_execution(task).await.unwrap();
            }

            i
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result < 5);
    }

    let executions = store.list_workflow_executions().await.unwrap();
    assert_eq!(executions.len(), 5);

    for i in 0..5 {
        let tasks = store
            .get_tasks_for_workflow(&format!("exec_{}", i))
            .await
            .unwrap();
        assert_eq!(tasks.len(), 3);
    }
}

#[tokio::test]
async fn test_wal_mode_concurrency() {
    let store = Arc::new(create_test_store().await);

    let reader_store = Arc::clone(&store);
    let writer_store = Arc::clone(&store);

    let reader_handle = task::spawn(async move {
        let mut read_count = 0;
        for _ in 0..100 {
            let workflows = reader_store.list_workflows().await.unwrap();
            read_count += workflows.len();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
        read_count
    });

    let writer_handle = task::spawn(async move {
        for i in 0..50 {
            let workflow = WorkflowDefinition {
                id: format!("wal_test_{}", i),
                name: format!("WAL Test {}", i),
                content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
                ..Default::default()
            };

            writer_store.save_workflow(&workflow).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        }
    });

    writer_handle.await.unwrap();
    let reader_result = reader_handle.await.unwrap();

    assert!(reader_result > 0);

    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 50);
}
