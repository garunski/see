//! Tests for concurrency and multi-process access
//!
//! Tests multiple concurrent readers, write during read following Single Responsibility Principle.

use persistence::{
    Store, TaskExecution, TaskStatus, WorkflowDefinition, WorkflowExecution, WorkflowStatus,
};
use std::sync::Arc;
use tokio::task;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

#[tokio::test]
async fn test_concurrent_readers() {
    let store = Arc::new(create_test_store().await);

    // Add some test data
    let workflow = WorkflowDefinition {
        id: "concurrent-test".to_string(),
        name: "Concurrent Test Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };

    store.save_workflow(&workflow).await.unwrap();

    // Spawn multiple concurrent readers
    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = task::spawn(async move {
            // Each reader performs multiple read operations
            for _j in 0..5 {
                let workflows = store_clone.list_workflows().await.unwrap();
                assert_eq!(workflows.len(), 1);
                assert_eq!(workflows[0].id, "concurrent-test");

                let retrieved = store_clone.get_workflow("concurrent-test").await.unwrap();
                assert!(retrieved.is_some());
                assert_eq!(retrieved.unwrap().name, "Concurrent Test Workflow");

                // Small delay to simulate processing
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
            format!("reader_{}", i)
        });
        handles.push(handle);
    }

    // Wait for all readers to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.starts_with("reader_"));
    }
}

#[tokio::test]
async fn test_concurrent_writers() {
    let store = Arc::new(create_test_store().await);

    // Spawn multiple concurrent writers
    let mut handles = vec![];

    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = task::spawn(async move {
            // Each writer creates multiple workflows
            for j in 0..3 {
                let workflow = WorkflowDefinition {
                    id: format!("workflow_{}_{}", i, j),
                    name: format!("Workflow {}-{}", i, j),
                    content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
                    ..Default::default()
                };

                let result = store_clone.save_workflow(&workflow).await;
                assert!(result.is_ok());

                // Small delay to simulate processing
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
            i
        });
        handles.push(handle);
    }

    // Wait for all writers to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result < 5);
    }

    // Verify all workflows were created
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 15); // 5 writers * 3 workflows each
}

#[tokio::test]
async fn test_read_during_write() {
    let store = Arc::new(create_test_store().await);

    // Add initial data
    let initial_workflow = WorkflowDefinition {
        id: "initial".to_string(),
        name: "Initial Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        ..Default::default()
    };

    store.save_workflow(&initial_workflow).await.unwrap();

    // Spawn writer task
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

            // Small delay between writes
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });

    // Spawn reader task that reads during writes
    let reader_store = Arc::clone(&store);
    let reader_handle = task::spawn(async move {
        let mut read_count = 0;
        while read_count < 50 {
            let workflows = reader_store.list_workflows().await.unwrap();
            assert!(workflows.len() >= 1); // At least the initial workflow

            let retrieved = reader_store.get_workflow("initial").await.unwrap();
            assert!(retrieved.is_some());

            read_count += 1;

            // Small delay between reads
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
        read_count
    });

    // Wait for both tasks to complete
    writer_handle.await.unwrap();
    let reader_result = reader_handle.await.unwrap();

    assert_eq!(reader_result, 50);

    // Verify final state
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 11); // 1 initial + 10 writer workflows
}

#[tokio::test]
async fn test_concurrent_execution_operations() {
    let store = Arc::new(create_test_store().await);

    // Spawn multiple tasks that create executions and tasks concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = task::spawn(async move {
            // Create execution
            let execution = WorkflowExecution {
                id: format!("exec_{}", i),
                workflow_name: format!("Workflow {}", i),
                status: WorkflowStatus::Running,
                tasks: vec![],
                ..Default::default()
            };

            store_clone
                .save_workflow_execution(execution)
                .await
                .unwrap();

            // Create tasks for this execution
            for j in 0..3 {
                let task = TaskExecution {
                    id: format!("task_{}_{}", i, j),
                    workflow_id: format!("exec_{}", i),
                    name: format!("Task {}-{}", i, j),
                    status: TaskStatus::Complete,
                    ..Default::default()
                };

                store_clone.save_task_execution(task).await.unwrap();
            }

            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result < 5);
    }

    // Verify all executions and tasks were created
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

    // Test that WAL mode allows concurrent reads and writes
    let reader_store = Arc::clone(&store);
    let writer_store = Arc::clone(&store);

    // Start continuous reader
    let reader_handle = task::spawn(async move {
        let mut read_count = 0;
        for _ in 0..100 {
            let workflows = reader_store.list_workflows().await.unwrap();
            read_count += workflows.len();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
        read_count
    });

    // Start continuous writer
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

    // Wait for both tasks
    writer_handle.await.unwrap();
    let reader_result = reader_handle.await.unwrap();

    assert!(reader_result > 0); // Should have read some workflows

    // Verify final state
    let workflows = store.list_workflows().await.unwrap();
    assert_eq!(workflows.len(), 50);
}
