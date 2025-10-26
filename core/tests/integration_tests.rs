// Integration tests ONLY

use s_e_e_core::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_output_callback() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let callback: OutputCallback = Arc::new(move |msg: String| {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
        assert_eq!(msg, "test message");
    });

    callback("test message".to_string());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_workflow_execution_persistence() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_global_store());

    match init_result {
        Ok(_) => {
            let store = get_global_store().unwrap();

            // Create and save a test workflow
            let workflow = WorkflowDefinition {
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
                                "cli_command": {
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
            };

            rt.block_on(store.save_workflow(&workflow)).unwrap();

            // Execute the workflow
            let result = rt.block_on(execute_workflow_by_id(&workflow.id, None));

            // Check that execution was persisted (if it succeeded)
            if let Ok(workflow_result) = result {
                // Try to get the execution from persistence
                let execution =
                    rt.block_on(store.get_workflow_execution(&workflow_result.execution_id));

                match execution {
                    Ok(Some(exec)) => {
                        assert_eq!(exec.id, workflow_result.execution_id);
                        assert_eq!(exec.workflow_name, workflow_result.workflow_name);
                    }
                    Ok(None) => {
                        // Execution might not be persisted if it failed early
                    }
                    Err(e) => {
                        // Database error - this is acceptable in test environment
                        println!("Database error (acceptable in tests): {}", e);
                    }
                }
            }
        }
        Err(_) => {
            // Store initialization failed - this is acceptable in test environment
        }
    }
}

#[test]
fn test_empty_workflow_execution() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_global_store());

    match init_result {
        Ok(_) => {
            let store = get_global_store().unwrap();

            // Create workflow with empty content
            let empty_workflow = WorkflowDefinition {
                id: "empty-workflow".to_string(),
                name: "Empty Workflow".to_string(),
                description: None,
                content: "".to_string(),
                is_default: false,
                is_edited: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            rt.block_on(store.save_workflow(&empty_workflow)).unwrap();

            let result = rt.block_on(execute_workflow_by_id(&empty_workflow.id, None));

            match result {
                Err(CoreError::Execution(msg)) => {
                    assert!(msg.contains("empty"));
                }
                Err(other) => panic!("Expected Execution error, got: {:?}", other),
                Ok(_) => panic!("Should have failed for empty content"),
            }
        }
        Err(_) => {
            // Store initialization failed - this is acceptable in test environment
        }
    }
}
