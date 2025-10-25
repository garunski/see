//! Task operation tests

use crate::tests::common::{setup_test_db, create_test_workflow, create_test_task};
use crate::db::schema::initialize_schema;
use crate::operations::{WorkflowOperations, TaskOperations};

#[tokio::test]
async fn test_create_task() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let workflow_ops = WorkflowOperations::new(std::sync::Arc::new(pool.clone()));
    let task_ops = TaskOperations::new(std::sync::Arc::new(pool));
    
    // Create workflow first
    let workflow = create_test_workflow();
    workflow_ops.create_workflow(workflow.clone()).await.unwrap();
    
    // Create task
    let task = create_test_task(workflow.id.clone());
    task_ops.create_task(task.clone()).await.unwrap();
    
    // Verify task was created
    let retrieved = task_ops.get_task(&task.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, task.id);
    assert_eq!(retrieved.workflow_id, workflow.id);
}

#[tokio::test]
async fn test_get_tasks_for_workflow() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let workflow_ops = WorkflowOperations::new(std::sync::Arc::new(pool.clone()));
    let task_ops = TaskOperations::new(std::sync::Arc::new(pool));
    
    // Create workflow
    let workflow = create_test_workflow();
    workflow_ops.create_workflow(workflow.clone()).await.unwrap();
    
    // Create multiple tasks
    for i in 0..3 {
        let mut task = create_test_task(workflow.id.clone());
        task.task_name = format!("Task {}", i);
        task.execution_order = i as i32;
        task_ops.create_task(task).await.unwrap();
    }
    
    // Get tasks for workflow
    let tasks = task_ops.get_tasks_for_workflow(&workflow.id).await.unwrap();
    assert_eq!(tasks.len(), 3);
    
    // Verify ordering
    for (i, task) in tasks.iter().enumerate() {
        assert_eq!(task.execution_order, i as i32);
    }
}
