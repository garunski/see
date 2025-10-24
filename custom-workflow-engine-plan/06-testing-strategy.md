# Testing Strategy for Custom Workflow Engine

## Overview

This document outlines a comprehensive testing strategy for the custom workflow engine, ensuring reliability, performance, and feature completeness.

## Testing Goals

- **Reliability**: Ensure engine works correctly under all conditions
- **Performance**: Validate performance improvements over dataflow-rs
- **Feature Completeness**: Verify all features work as expected
- **Regression Prevention**: Catch issues before they reach production
- **User Experience**: Ensure smooth operation for end users

## Testing Pyramid

### 1. Unit Tests (70%)
- Individual component testing
- Fast execution
- High coverage
- Isolated testing

### 2. Integration Tests (20%)
- Component interaction testing
- Database integration
- API integration
- Medium execution time

### 3. End-to-End Tests (10%)
- Complete workflow testing
- User scenario testing
- Slow execution
- High confidence

## Unit Testing Strategy

### 1. Core Engine Components

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_dependency_resolution() {
        let tasks = vec![
            Task {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand { command: "echo".to_string(), args: vec!["hello".to_string()] },
                status: TaskStatus::Pending,
            },
            Task {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CliCommand { command: "echo".to_string(), args: vec!["world".to_string()] },
                status: TaskStatus::Pending,
            },
        ];
        
        let dependency_graph = DependencyGraph::new(tasks);
        let ready_tasks = dependency_graph.get_ready_tasks(&HashSet::new(), &HashSet::new());
        
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, "task1");
    }
    
    #[tokio::test]
    async fn test_parallel_task_execution() {
        let engine = CustomWorkflowEngine::new();
        let tasks = vec![
            Task {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand { command: "echo".to_string(), args: vec!["hello".to_string()] },
                status: TaskStatus::Pending,
            },
            Task {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand { command: "echo".to_string(), args: vec!["world".to_string()] },
                status: TaskStatus::Pending,
            },
        ];
        
        let start_time = Instant::now();
        let results = engine.execute_parallel_tasks(tasks).await.unwrap();
        let execution_time = start_time.elapsed();
        
        assert_eq!(results.len(), 2);
        assert!(execution_time < Duration::from_secs(2)); // Should be fast due to parallel execution
    }
    
    #[tokio::test]
    async fn test_pause_resume_functionality() {
        let engine = CustomWorkflowEngine::new();
        let context = Arc::new(Mutex::new(ExecutionContext::new(vec![], None, None, "test".to_string(), "test".to_string())));
        
        // Test pause
        let result = engine.pause_workflow("test_execution", "test_task", "Test pause").await;
        assert!(result.is_ok());
        
        // Test resume
        let result = engine.resume_workflow("test_execution").await;
        assert!(result.is_ok());
    }
}
```

### 2. Task Handler Testing

```rust
#[cfg(test)]
mod handler_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cli_command_handler() {
        let handler = CliCommandHandler::new();
        let context = Arc::new(Mutex::new(ExecutionContext::new(vec![], None, None, "test".to_string(), "test".to_string())));
        
        let task = Task {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            dependencies: vec![],
            function: TaskFunction::CliCommand {
                command: "echo".to_string(),
                args: vec!["hello world".to_string()],
            },
            status: TaskStatus::Pending,
        };
        
        let result = handler.execute(&context, &task).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(matches!(result, TaskResult::Completed(_)));
    }
    
    #[tokio::test]
    async fn test_cursor_agent_handler() {
        let handler = CursorAgentHandler::new();
        let context = Arc::new(Mutex::new(ExecutionContext::new(vec![], None, None, "test".to_string(), "test".to_string())));
        
        let task = Task {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            dependencies: vec![],
            function: TaskFunction::CursorAgent {
                prompt: "Test prompt".to_string(),
                config: json!({}),
            },
            status: TaskStatus::Pending,
        };
        
        let result = handler.execute(&context, &task).await;
        // Note: This might fail in test environment without cursor-agent
        // Should be mocked or skipped in CI
    }
}
```

### 3. Error Handling Testing

```rust
#[cfg(test)]
mod error_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_failure_handling() {
        let engine = CustomWorkflowEngine::new();
        let context = Arc::new(Mutex::new(ExecutionContext::new(vec![], None, None, "test".to_string(), "test".to_string())));
        
        let task = Task {
            id: "failing_task".to_string(),
            name: "Failing Task".to_string(),
            dependencies: vec![],
            function: TaskFunction::CliCommand {
                command: "nonexistent_command".to_string(),
                args: vec![],
            },
            status: TaskStatus::Pending,
        };
        
        let result = engine.execute_task(&context, &task).await;
        assert!(result.is_err());
        
        // Check that task status is updated to Failed
        let context_guard = context.lock().unwrap();
        let task_status = context_guard.get_task_status("failing_task");
        assert_eq!(task_status, Some(TaskStatus::Failed));
    }
    
    #[tokio::test]
    async fn test_retry_mechanism() {
        let engine = CustomWorkflowEngine::new();
        let retry_config = RetryConfig {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_retry_delay: Duration::from_secs(1),
            retry_on_errors: vec!["timeout".to_string()],
        };
        
        let context = Arc::new(Mutex::new(ExecutionContext::new(vec![], None, None, "test".to_string(), "test".to_string())));
        let task = Task {
            id: "retry_task".to_string(),
            name: "Retry Task".to_string(),
            dependencies: vec![],
            function: TaskFunction::CliCommand {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            },
            status: TaskStatus::Pending,
        };
        
        let start_time = Instant::now();
        let result = engine.execute_task_with_retry(&context, &task, &retry_config).await;
        let execution_time = start_time.elapsed();
        
        assert!(result.is_ok());
        // Should complete quickly since command will succeed
        assert!(execution_time < Duration::from_secs(1));
    }
}
```

## Integration Testing Strategy

### 1. Database Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_persistence() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_test_workflow();
        
        // Execute workflow
        let result = engine.execute_workflow(workflow.clone()).await.unwrap();
        
        // Check that workflow state is persisted
        let persisted_workflow = engine.get_workflow_execution(&result.execution_id).await.unwrap();
        assert_eq!(persisted_workflow.workflow.id, workflow.id);
        assert_eq!(persisted_workflow.state, WorkflowState::Completed);
    }
    
    #[tokio::test]
    async fn test_pause_resume_persistence() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_pause_workflow();
        
        // Execute workflow (should pause)
        let result = engine.execute_workflow(workflow.clone()).await.unwrap();
        assert!(matches!(result.state, WorkflowState::Paused { .. }));
        
        // Resume workflow
        let resume_result = engine.resume_workflow(&result.execution_id).await.unwrap();
        assert!(matches!(resume_result.state, WorkflowState::Completed));
    }
}
```

### 2. API Integration Tests

```rust
#[cfg(test)]
mod api_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_execution_api() {
        let app = create_test_app().await;
        let client = reqwest::Client::new();
        
        // Test workflow execution
        let response = client
            .post("http://localhost:8080/api/workflows/execute")
            .json(&json!({
                "workflow_id": "test_workflow",
                "parameters": {}
            }))
            .send()
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200);
        
        let result: WorkflowResult = response.json().await.unwrap();
        assert!(result.success);
    }
    
    #[tokio::test]
    async fn test_pause_resume_api() {
        let app = create_test_app().await;
        let client = reqwest::Client::new();
        
        // Execute workflow that pauses
        let response = client
            .post("http://localhost:8080/api/workflows/execute")
            .json(&json!({
                "workflow_id": "pause_workflow",
                "parameters": {}
            }))
            .send()
            .await
            .unwrap();
        
        let result: WorkflowResult = response.json().await.unwrap();
        assert!(matches!(result.state, WorkflowState::Paused { .. }));
        
        // Resume workflow
        let resume_response = client
            .post(&format!("http://localhost:8080/api/workflows/{}/resume", result.execution_id))
            .send()
            .await
            .unwrap();
        
        assert_eq!(resume_response.status(), 200);
    }
}
```

## End-to-End Testing Strategy

### 1. Complete Workflow Tests

```rust
#[cfg(test)]
mod e2e_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_complete_user_input_workflow() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_user_input_workflow();
        
        // Execute workflow
        let result = engine.execute_workflow(workflow.clone()).await.unwrap();
        
        // Should pause for user input
        assert!(matches!(result.state, WorkflowState::Paused { .. }));
        
        // Simulate user input
        engine.submit_user_input(&result.execution_id, "user_choice", "yes").await.unwrap();
        
        // Resume workflow
        let final_result = engine.resume_workflow(&result.execution_id).await.unwrap();
        
        // Should complete successfully
        assert!(matches!(final_result.state, WorkflowState::Completed));
        assert!(final_result.success);
    }
    
    #[tokio::test]
    async fn test_parallel_execution_workflow() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_parallel_workflow();
        
        let start_time = Instant::now();
        let result = engine.execute_workflow(workflow.clone()).await.unwrap();
        let execution_time = start_time.elapsed();
        
        // Should complete successfully
        assert!(result.success);
        
        // Should be faster than sequential execution
        let sequential_time = estimate_sequential_time(&workflow);
        assert!(execution_time < sequential_time);
    }
}
```

### 2. Performance Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_comparison() {
        let dataflow_engine = DataflowEngineAdapter::new();
        let custom_engine = CustomWorkflowEngine::new();
        let workflow = create_performance_test_workflow();
        
        // Test dataflow-rs performance
        let dataflow_start = Instant::now();
        let dataflow_result = dataflow_engine.execute_workflow(workflow.clone()).await.unwrap();
        let dataflow_time = dataflow_start.elapsed();
        
        // Test custom engine performance
        let custom_start = Instant::now();
        let custom_result = custom_engine.execute_workflow(workflow.clone()).await.unwrap();
        let custom_time = custom_start.elapsed();
        
        // Custom engine should be faster
        assert!(custom_time < dataflow_time);
        
        // Results should be equivalent
        assert_eq!(dataflow_result.success, custom_result.success);
        assert_eq!(dataflow_result.task_count, custom_result.task_count);
    }
    
    #[tokio::test]
    async fn test_memory_usage() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_memory_test_workflow();
        
        let initial_memory = get_memory_usage();
        
        // Execute workflow
        let result = engine.execute_workflow(workflow).await.unwrap();
        
        let final_memory = get_memory_usage();
        let memory_increase = final_memory - initial_memory;
        
        // Memory increase should be reasonable
        assert!(memory_increase < 100 * 1024 * 1024); // 100MB
    }
}
```

## Load Testing Strategy

### 1. Concurrent Workflow Execution

```rust
#[cfg(test)]
mod load_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_concurrent_workflow_execution() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_test_workflow();
        
        // Execute 100 workflows concurrently
        let mut handles = Vec::new();
        for i in 0..100 {
            let engine_clone = engine.clone();
            let workflow_clone = workflow.clone();
            let handle = tokio::spawn(async move {
                engine_clone.execute_workflow(workflow_clone).await
            });
            handles.push(handle);
        }
        
        // Wait for all workflows to complete
        let results = futures::future::join_all(handles).await;
        
        // All workflows should succeed
        for result in results {
            assert!(result.is_ok());
            let workflow_result = result.unwrap();
            assert!(workflow_result.is_ok());
        }
    }
    
    #[tokio::test]
    async fn test_high_frequency_execution() {
        let engine = CustomWorkflowEngine::new();
        let workflow = create_test_workflow();
        
        let start_time = Instant::now();
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Execute workflows for 30 seconds
        while start_time.elapsed() < Duration::from_secs(30) {
            match engine.execute_workflow(workflow.clone()).await {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        // Success rate should be high
        let total_executions = success_count + error_count;
        let success_rate = success_count as f64 / total_executions as f64;
        assert!(success_rate > 0.95); // 95% success rate
    }
}
```

## Test Data Management

### 1. Test Workflow Definitions

```rust
fn create_test_workflow() -> Workflow {
    Workflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![
            Task {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                },
                status: TaskStatus::Pending,
            },
            Task {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["world".to_string()],
                },
                status: TaskStatus::Pending,
            },
        ],
    }
}

fn create_pause_workflow() -> Workflow {
    Workflow {
        id: "pause_workflow".to_string(),
        name: "Pause Workflow".to_string(),
        tasks: vec![
            Task {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: Some(PauseConfig {
                    prompt: "Continue?".to_string(),
                    variable: "continue".to_string(),
                    input_type: InputType::YesNo,
                }),
            },
        ],
    }
}
```

### 2. Test Database Setup

```rust
async fn setup_test_database() -> Arc<dyn AuditStore> {
    let store = create_test_store().await;
    
    // Create test tables
    store.create_tables().await.unwrap();
    
    // Insert test data
    store.insert_test_data().await.unwrap();
    
    store
}
```

## Continuous Integration

### 1. GitHub Actions Workflow

```yaml
name: Test Custom Workflow Engine

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run unit tests
      run: cargo test --lib
    
    - name: Run integration tests
      run: cargo test --test integration_tests
    
    - name: Run end-to-end tests
      run: cargo test --test e2e_tests
    
    - name: Run performance tests
      run: cargo test --test performance_tests
    
    - name: Run load tests
      run: cargo test --test load_tests
```

### 2. Test Coverage

```yaml
- name: Generate test coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Html --output-dir coverage/
    
- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    file: coverage/cobertura.xml
```

## Test Monitoring

### 1. Test Metrics

- **Test Coverage**: Target 90%+ code coverage
- **Test Execution Time**: Unit tests < 1s, integration tests < 10s
- **Test Reliability**: 99%+ test pass rate
- **Performance Regression**: No performance degradation

### 2. Test Reporting

- **Test Results**: Detailed test reports
- **Coverage Reports**: HTML coverage reports
- **Performance Reports**: Benchmark comparisons
- **Failure Analysis**: Root cause analysis for failures

## Conclusion

This comprehensive testing strategy ensures the custom workflow engine is reliable, performant, and feature-complete. The multi-layered approach provides confidence in the engine's correctness while maintaining fast feedback loops for development.
