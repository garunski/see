//! Multi-instance tests

use crate::tests::common::setup_test_db;
use crate::db::schema::initialize_schema;
use crate::instance::{InstanceManager, MultiInstanceCoordinator};

#[tokio::test]
async fn test_instance_isolation() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let pool = std::sync::Arc::new(pool);
    
    // Create two instance managers
    let instance1 = InstanceManager::new(pool.clone());
    let instance2 = InstanceManager::new(pool.clone());
    
    assert_ne!(instance1.get_instance_id(), instance2.get_instance_id());
    
    // Create workflows for each instance
    let mut workflow1 = crate::models::WorkflowExecution::new("Instance 1 Workflow".to_string());
    let mut workflow2 = crate::models::WorkflowExecution::new("Instance 2 Workflow".to_string());
    
    instance1.create_workflow(workflow1.clone()).await.unwrap();
    instance2.create_workflow(workflow2.clone()).await.unwrap();
    
    // Each instance should only see its own workflows
    let instance1_workflows = instance1.get_workflows_for_instance().await.unwrap();
    let instance2_workflows = instance2.get_workflows_for_instance().await.unwrap();
    
    assert_eq!(instance1_workflows.len(), 1);
    assert_eq!(instance2_workflows.len(), 1);
    assert_eq!(instance1_workflows[0].workflow_name, "Instance 1 Workflow");
    assert_eq!(instance2_workflows[0].workflow_name, "Instance 2 Workflow");
}

#[tokio::test]
async fn test_cross_instance_visibility() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let pool = std::sync::Arc::new(pool);
    
    let instance1 = InstanceManager::new(pool.clone());
    let instance2 = InstanceManager::new(pool.clone());
    
    // Create workflows for each instance
    let mut workflow1 = crate::models::WorkflowExecution::new("Instance 1 Workflow".to_string());
    let mut workflow2 = crate::models::WorkflowExecution::new("Instance 2 Workflow".to_string());
    
    instance1.create_workflow(workflow1.clone()).await.unwrap();
    instance2.create_workflow(workflow2.clone()).await.unwrap();
    
    // Both instances should see all workflows
    let all_workflows1 = instance1.get_all_workflows().await.unwrap();
    let all_workflows2 = instance2.get_all_workflows().await.unwrap();
    
    assert_eq!(all_workflows1.len(), 2);
    assert_eq!(all_workflows2.len(), 2);
}

#[tokio::test]
async fn test_instance_coordinator() {
    let (_temp_dir, pool) = setup_test_db();
    let conn = pool.get_connection().unwrap();
    initialize_schema(&conn).unwrap();
    drop(conn);
    
    let pool = std::sync::Arc::new(pool);
    let coordinator = MultiInstanceCoordinator::new(pool.clone());
    
    // Create an instance and some workflows
    let instance = InstanceManager::new(pool);
    for i in 0..3 {
        let mut workflow = crate::models::WorkflowExecution::new(format!("Workflow {}", i));
        instance.create_workflow(workflow).await.unwrap();
    }
    
    // Get instance stats
    let stats = coordinator.get_instance_stats().await.unwrap();
    assert_eq!(stats.total_workflows, 3);
    assert_eq!(stats.active_instances, 1);
}
