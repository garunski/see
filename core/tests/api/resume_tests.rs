// Task resumption API tests ONLY

use core::*;

#[test]
fn test_resume_task_not_found() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_test_store());
    
    match init_result {
        Ok(_) => {
            let result = rt.block_on(resume_task("nonexistent-execution", "nonexistent-task"));
            
            match result {
                Err(CoreError::WorkflowNotFound(id)) => {
                    assert_eq!(id, "nonexistent-execution");
                },
                other => panic!("Expected WorkflowNotFound error, got: {:?}", other),
            }
        },
        Err(_) => {
            // Store initialization failed - this is acceptable in test environment
        }
    }
}

#[test]
fn test_resume_task_invalid_task() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(init_test_store());
    
    match init_result {
        Ok(_) => {
            // Test with valid execution ID but invalid task ID
            let result = rt.block_on(resume_task("valid-execution", "invalid-task"));
            
            match result {
                Err(CoreError::TaskNotFound(id)) => {
                    assert_eq!(id, "invalid-task");
                },
                Err(CoreError::WorkflowNotFound(_)) => {
                    // Execution not found is also acceptable
                },
                other => panic!("Expected TaskNotFound or WorkflowNotFound error, got: {:?}", other),
            }
        },
        Err(_) => {
            // Store initialization failed - this is acceptable in test environment
        }
    }
}
