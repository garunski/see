use s_e_e_core::{
    execute_workflow_by_id, get_global_store, init_test_store, CoreError, OutputCallback,
    WorkflowDefinition,
};
use serial_test::serial;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn create_test_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Workflow".to_string(),
        description: Some("Test description".to_string()),
        content: r#"{
            "id": "test-workflow",
            "name": "Test Workflow",
            "tasks": [
                {
                    "id": "task-1",
                    "name": "Echo Hello",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Hello, World!"]
                        }
                    },
                    "next_tasks": []
                }
            ]
        }"#
        .to_string(),
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[test]
#[serial]
fn test_workflow_execution_flow() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_test_store());
    init_result.expect("Failed to initialize test store");

    let store = get_global_store().unwrap();

    let workflow = create_test_workflow();
    rt.block_on(store.save_workflow(&workflow)).unwrap();

    let result = rt.block_on(execute_workflow_by_id(&workflow.id, None));

    match result {
        Ok(workflow_result) => {
            assert_eq!(workflow_result.workflow_name, "Test Workflow");
            assert!(!workflow_result.execution_id.is_empty());
        }
        Err(CoreError::Engine(_)) => {}
        Err(other) => {
            panic!("Unexpected error: {:?}", other);
        }
    }
}

#[test]
#[serial]
fn test_workflow_not_found() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_test_store());
    init_result.expect("Failed to initialize test store");

    let result = rt.block_on(execute_workflow_by_id("nonexistent-workflow", None));

    match result {
        Err(CoreError::WorkflowNotFound(id)) => {
            assert_eq!(id, "nonexistent-workflow");
        }
        other => panic!("Expected WorkflowNotFound error, got: {:?}", other),
    }
}

#[test]
#[serial]
fn test_workflow_execution_with_callback() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_test_store());
    init_result.expect("Failed to initialize test store");

    let store = get_global_store().unwrap();

    let workflow = create_test_workflow();
    rt.block_on(store.save_workflow(&workflow)).unwrap();

    let callback_called = Arc::new(AtomicBool::new(false));
    let callback_called_clone = callback_called.clone();

    let callback: OutputCallback = Arc::new(move |_msg: String| {
        callback_called_clone.store(true, Ordering::SeqCst);
    });

    let _result = rt.block_on(execute_workflow_by_id(&workflow.id, Some(callback)));
}

#[test]
#[serial]
fn test_invalid_workflow_execution() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_test_store());
    init_result.expect("Failed to initialize test store");

    let store = get_global_store().unwrap();

    let invalid_workflow = WorkflowDefinition {
        id: "invalid-workflow".to_string(),
        name: "Invalid Workflow".to_string(),
        description: None,
        content: "invalid json".to_string(),
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    rt.block_on(store.save_workflow(&invalid_workflow)).unwrap();

    let result = rt.block_on(execute_workflow_by_id(&invalid_workflow.id, None));

    match result {
        Err(CoreError::Execution(_)) => {}
        Err(other) => panic!("Expected Engine error, got: {:?}", other),
        Ok(_) => panic!("Should have failed for invalid JSON"),
    }
}
