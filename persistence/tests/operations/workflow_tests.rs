//! Workflow operation tests

use crate::tests::common::{setup_test_db, create_test_workflow};
use crate::db::schema::initialize_schema;
use crate::operations::WorkflowOperations;

#[tokio::test]
async fn test_create_workflow() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let workflow_ops = WorkflowOperations::new(std::sync::Arc::new(pool));
    let workflow = create_test_workflow();
    
    workflow_ops.create_workflow(workflow.clone()).await.unwrap();
    
    // Verify workflow was created by getting it
    let retrieved = workflow_ops.get_workflow(&workflow.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, workflow.id);
    assert_eq!(retrieved.workflow_name, "Test Workflow");
}

#[tokio::test]
async fn test_get_workflow_not_found() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let workflow_ops = WorkflowOperations::new(std::sync::Arc::new(pool));
    let result = workflow_ops.get_workflow("nonexistent-id").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_workflow_status() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let workflow_ops = WorkflowOperations::new(std::sync::Arc::new(pool));
    let mut workflow = create_test_workflow();
    
    // Create workflow
    workflow_ops.create_workflow(workflow.clone()).await.unwrap();
    
    // Update status
    workflow_ops.update_workflow_status(&workflow.id, "running").await.unwrap();
    
    // Verify update
    let retrieved = workflow_ops.get_workflow(&workflow.id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, "running");
}

#[tokio::test]
async fn test_list_workflows() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let workflow_ops = WorkflowOperations::new(std::sync::Arc::new(pool));
    
    // Create multiple workflows
    for i in 0..5 {
        let mut workflow = create_test_workflow();
        workflow.workflow_name = format!("Test Workflow {}", i);
        workflow_ops.create_workflow(workflow).await.unwrap();
    }
    
    // List workflows
    let workflows = workflow_ops.list_workflows(10, 0).await.unwrap();
    assert_eq!(workflows.len(), 5);
}
