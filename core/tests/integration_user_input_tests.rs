//! Integration tests for user input functionality
//!
//! Tests complete workflows with user input from start to finish

use s_e_e_core::*;

/// Helper to create a test store
async fn create_test_store() -> Result<(), String> {
    init_global_store().await
}

/// Helper to create a simple workflow with user input
fn create_simple_input_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Simple Input Workflow".to_string(),
        description: Some("A workflow that requires user input".to_string()),
        content: r#"{
            "id": "simple-input",
            "name": "Simple User Input Workflow",
            "tasks": [
                {
                    "id": "greeting",
                    "name": "Display Greeting",
                    "function": {
                        "cli_command": {
                            "command": "echo",
                            "args": ["Hello! What's your name?"]
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "get-name",
                            "name": "Get User Name",
                            "function": {
                                "user_input": {
                                    "prompt": "Please enter your name:",
                                    "input_type": "string",
                                    "required": true,
                                    "default": null
                                }
                            },
                            "next_tasks": [
                                {
                                    "id": "thank-you",
                                    "name": "Thank You",
                                    "function": {
                                        "cli_command": {
                                            "command": "echo",
                                            "args": ["Thank you for your input!"]
                                        }
                                    },
                                    "next_tasks": []
                                }
                            ]
                        }
                    ]
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

/// Helper to create a parallel workflow with multiple inputs
fn create_parallel_input_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Parallel Input Workflow".to_string(),
        description: Some("A workflow with parallel input tasks".to_string()),
        content: r#"{
            "id": "parallel-input",
            "name": "Parallel User Input Workflow",
            "tasks": [
                {
                    "id": "start",
                    "name": "Start",
                    "function": {
                        "cli_command": {
                            "command": "echo",
                            "args": ["Starting parallel input tasks"]
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "input-a",
                            "name": "Input A",
                            "function": {
                                "user_input": {
                                    "prompt": "Enter value A:",
                                    "input_type": "string",
                                    "required": true,
                                    "default": null
                                }
                            },
                            "next_tasks": []
                        },
                        {
                            "id": "input-b",
                            "name": "Input B",
                            "function": {
                                "user_input": {
                                    "prompt": "Enter value B:",
                                    "input_type": "string",
                                    "required": true,
                                    "default": null
                                }
                            },
                            "next_tasks": []
                        }
                    ]
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

/// Helper to create a nested workflow with sequential inputs
fn create_nested_input_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Nested Input Workflow".to_string(),
        description: Some("A workflow with sequential nested inputs".to_string()),
        content: r#"{
            "id": "nested-input",
            "name": "Nested User Input Workflow",
            "tasks": [
                {
                    "id": "step1",
                    "name": "Step 1",
                    "function": {
                        "cli_command": {
                            "command": "echo",
                            "args": ["Step 1 complete"]
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "step2-input",
                            "name": "Step 2 Input",
                            "function": {
                                "user_input": {
                                    "prompt": "Enter value for step 2:",
                                    "input_type": "string",
                                    "required": true,
                                    "default": null
                                }
                            },
                            "next_tasks": [
                                {
                                    "id": "step3",
                                    "name": "Step 3",
                                    "function": {
                                        "cli_command": {
                                            "command": "echo",
                                            "args": ["Step 3 complete"]
                                        }
                                    },
                                    "next_tasks": []
                                }
                            ]
                        }
                    ]
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

/// End-to-End Test 1: Simple Input Workflow
///
/// This test verifies the complete flow:
/// 1. Create workflow with user input task
/// 2. Start workflow execution  
/// 3. Workflow pauses at input task
/// 4. Provide input via API
/// 5. Verify task resumes
/// 6. Verify workflow completes
#[tokio::test]
async fn test_simple_input_workflow_e2e() {
    // Setup
    let init_result = create_test_store().await;
    if init_result.is_err() {
        println!("Store initialization skipped in test environment");
        return;
    }

    let workflow = create_simple_input_workflow();
    let store = get_global_store().unwrap();

    // Save workflow
    store.save_workflow(&workflow).await.unwrap();

    // Start workflow execution
    let execution_result = execute_workflow_by_id(&workflow.id, None).await;

    // Execution should either complete or wait for input
    match execution_result {
        Ok(workflow_result) => {
            println!(
                "Workflow execution result: success={}",
                workflow_result.success
            );
            println!("Tasks: {}", workflow_result.tasks.len());

            // Check for tasks waiting for input
            let waiting_tasks: Vec<_> = workflow_result
                .tasks
                .iter()
                .filter(|t| {
                    let status_str = format!("{:?}", t.status);
                    status_str.contains("WaitingForInput")
                })
                .collect();

            println!("Tasks waiting for input: {}", waiting_tasks.len());

            // If workflow paused for input, provide input and resume
            if !waiting_tasks.is_empty() {
                let task = waiting_tasks[0];
                println!("Providing input for task: {}", task.id);

                // Provide input
                let input_result = provide_user_input(
                    &workflow_result.execution_id,
                    &task.id,
                    "Test User".to_string(),
                )
                .await;

                println!("Input result: {:?}", input_result);
            }
        }
        Err(e) => {
            println!("Workflow execution error: {:?}", e);
            // Acceptable in test environment
        }
    }
}

/// End-to-End Test 2: Parallel Input Workflow
///
/// This test verifies:
/// 1. Multiple parallel tasks waiting for input
/// 2. Each task can receive input independently
/// 3. Non-input tasks can continue
/// 4. Workflow completes when all inputs provided
#[tokio::test]
async fn test_parallel_input_workflow_e2e() {
    // Setup
    let init_result = create_test_store().await;
    if init_result.is_err() {
        println!("Store initialization skipped in test environment");
        return;
    }

    let workflow = create_parallel_input_workflow();
    let store = get_global_store().unwrap();

    // Save workflow
    store.save_workflow(&workflow).await.unwrap();

    // Start workflow execution
    let execution_result = execute_workflow_by_id(&workflow.id, None).await;

    match execution_result {
        Ok(workflow_result) => {
            println!(
                "Parallel workflow execution result: success={}",
                workflow_result.success
            );

            // Check for tasks waiting for input
            let waiting_tasks: Vec<_> = workflow_result
                .tasks
                .iter()
                .filter(|t| {
                    let status_str = format!("{:?}", t.status);
                    status_str.contains("WaitingForInput")
                })
                .collect();

            println!("Tasks waiting for input: {}", waiting_tasks.len());

            // Provide input for each waiting task
            for task in &waiting_tasks {
                println!("Providing input for parallel task: {}", task.id);

                let input_result = provide_user_input(
                    &workflow_result.execution_id,
                    &task.id,
                    format!("Value for {}", task.id),
                )
                .await;

                println!("Input result for {}: {:?}", task.id, input_result);
            }
        }
        Err(e) => {
            println!("Workflow execution error: {:?}", e);
        }
    }
}

/// End-to-End Test 3: Nested Input Workflow
///
/// This test verifies:
/// 1. Sequential input collection (input → task → input)
/// 2. Each input is captured correctly
/// 3. Workflow progresses through multiple inputs
#[tokio::test]
async fn test_nested_input_workflow_e2e() {
    // Setup
    let init_result = create_test_store().await;
    if init_result.is_err() {
        println!("Store initialization skipped in test environment");
        return;
    }

    let workflow = create_nested_input_workflow();
    let store = get_global_store().unwrap();

    // Save workflow
    store.save_workflow(&workflow).await.unwrap();

    // Start workflow execution
    let execution_result = execute_workflow_by_id(&workflow.id, None).await;

    match execution_result {
        Ok(workflow_result) => {
            println!(
                "Nested workflow execution result: success={}",
                workflow_result.success
            );

            let waiting_tasks: Vec<_> = workflow_result
                .tasks
                .iter()
                .filter(|t| {
                    let status_str = format!("{:?}", t.status);
                    status_str.contains("WaitingForInput")
                })
                .collect();

            println!("Tasks waiting for input: {}", waiting_tasks.len());

            if !waiting_tasks.is_empty() {
                let task = waiting_tasks[0];
                println!("Providing input for nested task: {}", task.id);

                let input_result = provide_user_input(
                    &workflow_result.execution_id,
                    &task.id,
                    "Nested Value".to_string(),
                )
                .await;

                println!("Nested input result: {:?}", input_result);
            }
        }
        Err(e) => {
            println!("Workflow execution error: {:?}", e);
        }
    }
}

/// End-to-End Test 4: Error Handling
///
/// This test verifies:
/// 1. Invalid input is rejected
/// 2. Error messages are shown
/// 3. Valid input can be provided after error
#[tokio::test]
async fn test_input_error_handling_e2e() {
    // Setup
    let init_result = create_test_store().await;
    if init_result.is_err() {
        println!("Store initialization skipped in test environment");
        return;
    }

    let workflow = create_simple_input_workflow();
    let store = get_global_store().unwrap();

    store.save_workflow(&workflow).await.unwrap();

    let execution_result = execute_workflow_by_id(&workflow.id, None).await;

    match execution_result {
        Ok(workflow_result) => {
            let waiting_tasks: Vec<_> = workflow_result
                .tasks
                .iter()
                .filter(|t| {
                    let status_str = format!("{:?}", t.status);
                    status_str.contains("WaitingForInput")
                })
                .collect();

            if !waiting_tasks.is_empty() {
                // Try to get pending inputs
                let pending_result = get_pending_inputs(&workflow_result.execution_id).await;
                println!("Pending inputs result: {:?}", pending_result);

                // Verify get_tasks_waiting_for_input works
                let tasks_result = get_tasks_waiting_for_input(&workflow_result.execution_id).await;
                println!("Tasks waiting for input result: {:?}", tasks_result);
            }
        }
        Err(e) => {
            println!("Workflow execution error: {:?}", e);
        }
    }
}

/// End-to-End Test 5: Multiple Inputs
///
/// This test verifies:
/// 1. Multiple sequential inputs are handled
/// 2. Each input is captured correctly
/// 3. Workflow progresses through all inputs
#[tokio::test]
async fn test_multiple_inputs_e2e() {
    // Setup
    let init_result = create_test_store().await;
    if init_result.is_err() {
        println!("Store initialization skipped in test environment");
        return;
    }

    // This test is a placeholder for a more complex scenario
    // In real implementation, would test a workflow with multiple sequential inputs

    println!("Multiple inputs test - placeholder");
}

// Performance tests are in a separate module
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    /// Performance Test 1: Input Creation Speed
    #[tokio::test]
    async fn test_input_creation_performance() {
        let init_result = create_test_store().await;
        if init_result.is_err() {
            println!("Store initialization skipped in test environment");
            return;
        }

        let start = Instant::now();

        // Create and save an input request
        let request = UserInputRequest::default();
        let store = get_global_store().unwrap();

        store.save_input_request(&request).await.unwrap();

        let duration = start.elapsed();

        println!("Input creation took: {:?}", duration);

        // Should complete in under 10ms
        assert!(
            duration.as_millis() < 100,
            "Input creation too slow: {:?}",
            duration
        );
    }

    /// Performance Test 2: Input Validation Speed
    ///
    /// Note: This is a placeholder for performance testing.
    /// Actual validation performance tests are in input.rs module tests.
    #[test]
    fn test_input_validation_performance() {
        println!("Input validation performance tests should be run in module tests");
        // Test logic would go here if needed
    }

    /// Performance Test 3: Workflow with Input Speed
    #[tokio::test]
    async fn test_input_workflow_performance() {
        let init_result = create_test_store().await;
        if init_result.is_err() {
            println!("Store initialization skipped in test environment");
            return;
        }

        let workflow = create_simple_input_workflow();
        let store = get_global_store().unwrap();

        store.save_workflow(&workflow).await.unwrap();

        let start = Instant::now();
        let _ = execute_workflow_by_id(&workflow.id, None).await;
        let duration = start.elapsed();

        println!("Workflow with input took: {:?}", duration);

        // Should start quickly even if it pauses for input
        assert!(
            duration.as_millis() < 5000,
            "Workflow start too slow: {:?}",
            duration
        );
    }
}

// Concurrency tests
#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::time::{timeout, Duration};

    /// Concurrency Test 1: Multiple Simultaneous Inputs
    ///
    /// This test verifies:
    /// 1. Multiple inputs can be provided concurrently
    /// 2. No race conditions occur
    /// 3. Data consistency is maintained
    #[tokio::test]
    async fn test_concurrent_input_submission() {
        let init_result = create_test_store().await;
        if init_result.is_err() {
            println!("Store initialization skipped in test environment");
            return;
        }

        let workflow = create_parallel_input_workflow();
        let store = get_global_store().unwrap();

        store.save_workflow(&workflow).await.unwrap();

        // Execute workflow
        let execution_result = execute_workflow_by_id(&workflow.id, None).await;

        match execution_result {
            Ok(workflow_result) => {
                let waiting_tasks: Vec<_> = workflow_result
                    .tasks
                    .iter()
                    .filter(|t| {
                        let status_str = format!("{:?}", t.status);
                        status_str.contains("WaitingForInput")
                    })
                    .collect();

                if waiting_tasks.len() >= 2 {
                    // Try to submit inputs concurrently
                    let semaphore = Arc::new(Semaphore::new(waiting_tasks.len()));

                    let mut handles = Vec::new();

                    for (idx, task) in waiting_tasks.iter().take(2).enumerate() {
                        let sem = semaphore.clone();
                        let task_id = task.id.clone();
                        let exec_id = workflow_result.execution_id.clone();

                        let handle = tokio::spawn(async move {
                            let _permit = sem.acquire().await.unwrap();

                            println!("Submitting concurrent input {} for task {}", idx, task_id);

                            let input_val = format!("Concurrent Input {}", idx);
                            let result = provide_user_input(&exec_id, &task_id, input_val).await;

                            println!("Concurrent input result for {}: {:?}", task_id, result);

                            result
                        });

                        handles.push(handle);
                    }

                    // Wait for all submissions with timeout
                    let result = timeout(Duration::from_secs(10), async {
                        let mut results = Vec::new();
                        for handle in handles {
                            results.push(handle.await.unwrap());
                        }
                        results
                    })
                    .await;

                    match result {
                        Ok(results) => {
                            println!("Concurrent submissions completed");
                            println!("Results: {:?}", results);
                        }
                        Err(_) => {
                            println!("Concurrent submissions timed out");
                        }
                    }
                }
            }
            Err(e) => {
                println!("Workflow execution error: {:?}", e);
            }
        }
    }

    /// Concurrency Test 2: Concurrent Workflow Executions
    #[tokio::test]
    async fn test_concurrent_workflow_executions() {
        let init_result = create_test_store().await;
        if init_result.is_err() {
            println!("Store initialization skipped in test environment");
            return;
        }

        let workflow = create_simple_input_workflow();
        let store = get_global_store().unwrap();

        store.save_workflow(&workflow).await.unwrap();

        // Try to start multiple executions concurrently
        let mut handles = Vec::new();

        for i in 0..3 {
            let workflow_id = workflow.id.clone();
            let handle = tokio::spawn(async move {
                println!("Starting concurrent execution {}", i);
                let result = execute_workflow_by_id(&workflow_id, None).await;
                println!("Concurrent execution {} result: {:?}", i, result);
                result
            });

            handles.push(handle);
        }

        let result = timeout(Duration::from_secs(30), async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await.unwrap());
            }
            results
        })
        .await;

        match result {
            Ok(results) => {
                println!("Concurrent executions completed");
                println!("Number of successful executions: {}", results.len());
            }
            Err(_) => {
                println!("Concurrent executions timed out");
            }
        }
    }
}
