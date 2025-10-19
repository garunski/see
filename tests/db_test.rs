use simple_workflow_app::db::models::TaskInfo;
use simple_workflow_app::db::models::{WorkflowExecution, WorkflowExecutionSummary};
use simple_workflow_app::db::{AuditStore, RedbAuditStore};
use simple_workflow_app::AuditEntry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// Use a mutex to ensure tests run sequentially and don't interfere
static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn create_test_execution(id: &str, workflow_name: &str) -> WorkflowExecution {
    WorkflowExecution {
        id: id.to_string(),
        workflow_name: workflow_name.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        success: true,
        tasks: vec![
            TaskInfo {
                id: "task_1".to_string(),
                name: "Test Task 1".to_string(),
                status: "complete".to_string(),
            },
            TaskInfo {
                id: "task_2".to_string(),
                name: "Test Task 2".to_string(),
                status: "complete".to_string(),
            },
        ],
        audit_trail: vec![AuditEntry {
            task_id: "task_1".to_string(),
            status: "200".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            changes_count: 1,
        }],
        per_task_logs: {
            let mut logs = HashMap::new();
            logs.insert("task_1".to_string(), vec!["Task 1 output".to_string()]);
            logs.insert("task_2".to_string(), vec!["Task 2 output".to_string()]);
            logs
        },
        errors: vec![],
    }
}

fn create_test_db_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let test_db_path = temp_dir.join(format!("test_audit_{}.redb", uuid::Uuid::new_v4()));
    test_db_path
}

fn cleanup_test_db(db_path: &PathBuf) {
    let _ = fs::remove_file(db_path);
}

fn run_test<F>(test_fn: F)
where
    F: FnOnce(),
{
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    test_fn();
}

#[test]
fn test_database_initialization() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone());

        assert!(store.is_ok(), "Database should initialize successfully");

        cleanup_test_db(&db_path);
    });
}

#[test]
fn test_database_default_path() {
    run_test(|| {
        let default_path = RedbAuditStore::default_path();
        assert!(default_path.is_ok(), "Should get default path");

        let path = default_path.unwrap();
        assert!(path.to_string_lossy().contains(".see"));
        assert!(path.to_string_lossy().contains("audit.redb"));
    });
}

#[test]
fn test_save_and_retrieve_workflow_execution() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone()).unwrap();

        let execution = create_test_execution("test_exec_1", "Test Workflow");

        // Save execution
        let result = store.save_workflow_execution(&execution);
        assert!(result.is_ok(), "Should save execution successfully");
        assert_eq!(result.unwrap(), "test_exec_1");

        // Retrieve execution
        let retrieved = store.get_workflow_execution("test_exec_1");
        assert!(retrieved.is_ok(), "Should retrieve execution successfully");

        let retrieved_execution = retrieved.unwrap();
        assert_eq!(retrieved_execution.id, "test_exec_1");
        assert_eq!(retrieved_execution.workflow_name, "Test Workflow");
        assert_eq!(retrieved_execution.tasks.len(), 2);
        assert_eq!(retrieved_execution.per_task_logs.len(), 2);

        cleanup_test_db(&db_path);
    });
}

#[test]
fn test_list_workflow_executions() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone()).unwrap();

        // Save multiple executions
        let exec1 = create_test_execution("exec_1", "Workflow 1");
        let exec2 = create_test_execution("exec_2", "Workflow 2");
        let exec3 = create_test_execution("exec_3", "Workflow 3");

        store.save_workflow_execution(&exec1).unwrap();
        store.save_workflow_execution(&exec2).unwrap();
        store.save_workflow_execution(&exec3).unwrap();

        // List executions
        let executions = store.list_workflow_executions(10);
        assert!(executions.is_ok(), "Should list executions successfully");

        let executions = executions.unwrap();
        assert_eq!(executions.len(), 3, "Should have 3 executions");

        // Should be ordered by timestamp (newest first)
        assert_eq!(executions[0].id, "exec_3");
        assert_eq!(executions[1].id, "exec_2");
        assert_eq!(executions[2].id, "exec_1");

        // Test limit
        let limited = store.list_workflow_executions(2);
        assert!(limited.is_ok());
        assert_eq!(limited.unwrap().len(), 2);

        cleanup_test_db(&db_path);
    });
}

#[test]
fn test_delete_workflow_execution() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone()).unwrap();

        let execution = create_test_execution("test_delete", "Delete Test");

        // Save execution
        store.save_workflow_execution(&execution).unwrap();

        // Verify it exists
        let retrieved = store.get_workflow_execution("test_delete");
        assert!(retrieved.is_ok());

        // Delete execution
        let delete_result = store.delete_workflow_execution("test_delete");
        assert!(
            delete_result.is_ok(),
            "Should delete execution successfully"
        );

        // Verify it's gone
        let retrieved_after = store.get_workflow_execution("test_delete");
        assert!(
            retrieved_after.is_err(),
            "Should not find deleted execution"
        );

        // Verify it's not in the list
        let executions = store.list_workflow_executions(10).unwrap();
        assert!(!executions.iter().any(|e| e.id == "test_delete"));

        cleanup_test_db(&db_path);
    });
}

#[test]
fn test_get_nonexistent_execution() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone()).unwrap();

        let result = store.get_workflow_execution("nonexistent");
        assert!(result.is_err(), "Should error for nonexistent execution");

        cleanup_test_db(&db_path);
    });
}

#[test]
fn test_delete_nonexistent_execution() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone()).unwrap();

        let result = store.delete_workflow_execution("nonexistent");
        assert!(result.is_err(), "Should error for nonexistent execution");

        cleanup_test_db(&db_path);
    });
}

#[test]
fn test_execution_summary_fields() {
    run_test(|| {
        let db_path = create_test_db_path();
        let store = RedbAuditStore::new(db_path.clone()).unwrap();

        let mut execution = create_test_execution("summary_test", "Summary Test Workflow");
        execution.success = false; // Test failure case

        store.save_workflow_execution(&execution).unwrap();

        let executions = store.list_workflow_executions(1).unwrap();
        assert_eq!(executions.len(), 1);

        let summary = &executions[0];
        assert_eq!(summary.id, "summary_test");
        assert_eq!(summary.workflow_name, "Summary Test Workflow");
        assert_eq!(summary.success, false);
        assert_eq!(summary.task_count, 2);
        assert!(!summary.timestamp.is_empty());

        cleanup_test_db(&db_path);
    });
}
