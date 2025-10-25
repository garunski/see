//! Common test utilities

use tempfile::TempDir;
use crate::db::DatabasePool;

/// Set up a test database in a temporary directory
pub fn setup_test_db() -> (TempDir, DatabasePool) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let pool = DatabasePool::new(db_path.to_str().unwrap(), 5)
        .expect("Failed to create database pool");
    (temp_dir, pool)
}

/// Create a test workflow execution
pub fn create_test_workflow() -> crate::models::WorkflowExecution {
    crate::models::WorkflowExecution::new("Test Workflow".to_string())
}

/// Create a test task execution
pub fn create_test_task(workflow_id: String) -> crate::models::TaskExecution {
    crate::models::TaskExecution::new(workflow_id, "Test Task".to_string())
}
