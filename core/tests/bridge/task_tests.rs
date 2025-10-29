// Task conversion tests ONLY

use core::bridge::*;
use s_e_e_engine::{TaskInfo, TaskStatus as EngineTaskStatus};
use s_e_e_persistence::TaskStatus as PersistenceTaskStatus;
use std::collections::HashMap;

#[test]
fn test_task_status_conversion() {
    // Test all engine to persistence status conversions
    let test_cases = vec![
        (EngineTaskStatus::Pending, PersistenceTaskStatus::Pending),
        (EngineTaskStatus::InProgress, PersistenceTaskStatus::InProgress),
        (EngineTaskStatus::Complete, PersistenceTaskStatus::Complete),
        (EngineTaskStatus::Failed, PersistenceTaskStatus::Failed),
        (EngineTaskStatus::WaitingForInput, PersistenceTaskStatus::WaitingForInput),
    ];
    
    for (engine_status, expected_persistence_status) in test_cases {
        let task_info = TaskInfo {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            status: engine_status,
        };
        
        let task_execution = core::bridge::task::task_info_to_execution(
            &task_info,
            "workflow-1",
            &HashMap::new(),
            &vec![],
            chrono::Utc::now(),
            chrono::Utc::now(),
        );
        
        assert_eq!(task_execution.status, expected_persistence_status);
    }
}

#[test]
fn test_task_output_extraction() {
    let mut per_task_logs = HashMap::new();
    per_task_logs.insert("task-1".to_string(), vec!["line 1".to_string(), "line 2".to_string()]);
    per_task_logs.insert("task-2".to_string(), vec![]);
    
    let task_info = TaskInfo {
        id: "task-1".to_string(),
        name: "Test Task".to_string(),
        status: EngineTaskStatus::Complete,
    };
    
    let task_execution = core::bridge::task::task_info_to_execution(
        &task_info,
        "workflow-1",
        &per_task_logs,
        &vec![],
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    
    assert_eq!(task_execution.output, Some("line 1\nline 2".to_string()));
    
    // Test empty logs
    let task_info2 = TaskInfo {
        id: "task-2".to_string(),
        name: "Test Task 2".to_string(),
        status: EngineTaskStatus::Complete,
    };
    
    let task_execution2 = core::bridge::task::task_info_to_execution(
        &task_info2,
        "workflow-1",
        &per_task_logs,
        &vec![],
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    
    assert_eq!(task_execution2.output, None);
}

#[test]
fn test_task_error_extraction() {
    let errors = vec![
        "Task task-1 failed: command not found".to_string(),
        "Task task-2 failed: permission denied".to_string(),
        "General workflow error".to_string(),
    ];
    
    // Test task-specific error
    let task_info = TaskInfo {
        id: "task-1".to_string(),
        name: "Failed Task".to_string(),
        status: EngineTaskStatus::Failed,
    };
    
    let task_execution = core::bridge::task::task_info_to_execution(
        &task_info,
        "workflow-1",
        &HashMap::new(),
        &errors,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    
    assert_eq!(task_execution.error, Some("Task task-1 failed: command not found".to_string()));
    
    // Test task without specific error
    let task_info2 = TaskInfo {
        id: "task-3".to_string(),
        name: "Failed Task 3".to_string(),
        status: EngineTaskStatus::Failed,
    };
    
    let task_execution2 = core::bridge::task::task_info_to_execution(
        &task_info2,
        "workflow-1",
        &HashMap::new(),
        &errors,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    
    assert_eq!(task_execution2.error, Some("Task failed".to_string()));
    
    // Test successful task (no error)
    let task_info3 = TaskInfo {
        id: "task-4".to_string(),
        name: "Successful Task".to_string(),
        status: EngineTaskStatus::Complete,
    };
    
    let task_execution3 = core::bridge::task::task_info_to_execution(
        &task_info3,
        "workflow-1",
        &HashMap::new(),
        &errors,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    
    assert_eq!(task_execution3.error, None);
}
