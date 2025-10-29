// Execution conversion tests ONLY

use s_e_e_core::bridge::*;
use s_e_e_engine::WorkflowResult as EngineWorkflowResult;

#[test]
fn test_engine_result_to_core_result() {
    let engine_result = EngineWorkflowResult {
        success: true,
        workflow_name: "Test Workflow".to_string(),
        tasks: vec![],
        audit_trail: vec![],
        per_task_logs: std::collections::HashMap::new(),
        errors: vec![],
    };

    let execution_id = "exec-123".to_string();
    let core_result = workflow::engine_result_to_core_result(engine_result, execution_id.clone());

    assert!(core_result.success);
    assert_eq!(core_result.workflow_name, "Test Workflow");
    assert_eq!(core_result.execution_id, execution_id);
    assert_eq!(core_result.tasks.len(), 0);
    assert_eq!(core_result.audit_trail.len(), 0);
    assert_eq!(core_result.errors.len(), 0);
}

#[test]
fn test_engine_result_to_core_result_with_data() {
    use engine::{AuditEntry, AuditStatus, TaskInfo, TaskStatus};

    let tasks = vec![TaskInfo {
        id: "task-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Complete,
    }];

    let audit_trail = vec![AuditEntry {
        task_id: "task-1".to_string(),
        status: AuditStatus::Success,
        timestamp: "2024-01-15T10:30:45Z".to_string(),
        changes_count: 5,
        message: "Task completed".to_string(),
    }];

    let mut per_task_logs = std::collections::HashMap::new();
    per_task_logs.insert("task-1".to_string(), vec!["output line 1".to_string()]);

    let errors = vec!["Some error".to_string()];

    let engine_result = EngineWorkflowResult {
        success: false,
        workflow_name: "Failed Workflow".to_string(),
        tasks,
        audit_trail,
        per_task_logs,
        errors,
    };

    let execution_id = "exec-456".to_string();
    let core_result = workflow::engine_result_to_core_result(engine_result, execution_id.clone());

    assert!(!core_result.success);
    assert_eq!(core_result.workflow_name, "Failed Workflow");
    assert_eq!(core_result.execution_id, execution_id);
    assert_eq!(core_result.tasks.len(), 1);
    assert_eq!(core_result.audit_trail.len(), 1);
    assert_eq!(core_result.errors.len(), 1);
    assert_eq!(core_result.errors[0], "Some error");
}
