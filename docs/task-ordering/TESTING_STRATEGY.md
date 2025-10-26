# Task Ordering - Testing Strategy

## Overview

Comprehensive testing strategy for task ordering and workflow snapshot features.

## Test Layers

### 1. Unit Tests

### Persistence Layer Tests

**File**: `persistence/tests/models/execution_tests.rs`

Tests for `WorkflowExecution` model:

```rust
#[test]
fn test_workflow_execution_serialization_with_snapshot() {
    let execution = create_execution_with_snapshot();
    let json = serde_json::to_string(&execution).unwrap();
    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.workflow_snapshot, execution.workflow_snapshot);
}

#[test]
fn test_workflow_execution_default_includes_snapshot() {
    let execution = WorkflowExecution::default();
    assert!(execution.workflow_snapshot.is_object());
}
```

**File**: `persistence/tests/store/execution_tests.rs`

Tests for store operations:

```rust
#[tokio::test]
async fn test_save_and_retrieve_with_snapshot() {
    let store = create_test_store().await;
    let execution = create_execution_with_snapshot();
    
    store.save_workflow_execution(execution.clone()).await.unwrap();
    let retrieved = store.get_workflow_execution(&execution.id).await.unwrap().unwrap();
    
    assert_eq!(retrieved.workflow_snapshot, execution.workflow_snapshot);
}
```

### Core Layer Tests

**File**: `core/tests/api/execution_tests.rs`

Tests for execution API:

```rust
#[tokio::test]
async fn test_execute_workflow_stores_snapshot() {
    // Setup workflow with content
    let workflow = create_workflow_with_content();
    store.save_workflow(&workflow).await.unwrap();

    // Execute workflow
    let result = execute_workflow_by_id(&workflow.id, None).await.unwrap();

    // Verify snapshot stored
    let execution = store.get_workflow_execution(&result.execution_id)
        .await
        .unwrap()
        .unwrap();
    
    assert!(!execution.workflow_snapshot.is_null());
}

#[tokio::test]
async fn test_execution_preserves_snapshot_on_resume() {
    // Create paused execution
    let execution = create_paused_execution();
    store.save_workflow_execution(execution).await.unwrap();

    // Resume
    resume_task(&execution.id, &task_id).await.unwrap();

    // Verify snapshot still present
    let updated = store.get_workflow_execution(&execution.id)
        .await
        .unwrap()
        .unwrap();
    
    assert!(!updated.workflow_snapshot.is_null());
}
```

### Engine Layer Tests

**File**: `engine/tests/engine_tests.rs`

Test pause behavior:

```rust
#[tokio::test]
async fn test_workflow_pauses_for_user_input() {
    let workflow = create_workflow_with_user_input();
    let engine = WorkflowEngine::new();
    
    let result = engine.execute_workflow(workflow).await.unwrap();
    
    // Verify paused state
    assert!(result.tasks.iter().any(|t| t.status == TaskStatus::WaitingForInput));
}
```

## Integration Tests

### End-to-End Execution

**File**: `core/tests/execution_integration_tests.rs`

```rust
#[tokio::test]
async fn test_end_to_end_with_snapshot() {
    // 1. Create workflow
    let workflow = create_test_workflow();
    store.save_workflow(&workflow).await.unwrap();

    // 2. Execute
    let result = execute_workflow_by_id(&workflow.id, None).await.unwrap();

    // 3. Verify snapshot stored
    let execution = store.get_workflow_execution(&result.execution_id)
        .await
        .unwrap()
        .unwrap();
    
    assert!(!execution.workflow_snapshot.is_null());

    // 4. Verify task order
    let task_ids = extract_task_ids(&execution.workflow_snapshot);
    let actual_order: Vec<_> = execution.tasks.iter()
        .map(|t| t.id.clone())
        .collect();
    
    assert_eq!(actual_order, task_ids);
}
```

### User Input Workflow

**File**: `core/tests/integration_user_input_tests.rs`

```rust
#[tokio::test]
async fn test_user_input_workflow_pauses() {
    // Create workflow with user input task
    let workflow = create_user_input_workflow();
    store.save_workflow(&workflow).await.unwrap();

    // Execute
    let result = execute_workflow_by_id(&workflow.id, None).await.unwrap();

    // Verify paused
    assert!(!result.success);
    assert!(result.tasks.iter().any(|t| t.status == TaskStatus::WaitingForInput));

    // Verify execution stored as paused
    let execution = store.get_workflow_execution(&result.execution_id)
        .await
        .unwrap()
        .unwrap();
    
    assert_eq!(execution.status, WorkflowStatus::Running);
}
```

## Manual Testing

### Test Scenarios

#### Scenario 1: Simple Sequential Workflow

**Workflow**:
```json
{
  "tasks": [
    {"id": "task1", "next_tasks": [
      {"id": "task2", "next_tasks": []}
    ]}
  ]
}
```

**Steps**:
1. Execute workflow
2. Check GUI - tasks should display as: task1, task2
3. Verify correct order

#### Scenario 2: Parallel Workflow

**Workflow**:
```json
{
  "tasks": [
    {"id": "task1", "next_tasks": []},
    {"id": "task2", "next_tasks": []}
  ]
}
```

**Steps**:
1. Execute workflow
2. Check GUI - tasks should display as: task1, task2
3. Verify parallel execution

#### Scenario 3: User Input Workflow

**Workflow**:
```json
{
  "tasks": [
    {"id": "get_input", "function": {"user_input": {...}}, "next_tasks": [
      {"id": "use_input", "next_tasks": []}
    ]}
  ]
}
```

**Steps**:
1. Execute workflow
2. Verify pause at get_input
3. Provide input via GUI
4. Resume workflow
5. Verify completion
6. Check task order

### GUI Manual Testing

**Checklist**:

- [ ] Tasks display in correct order
- [ ] Task details panel shows correct task
- [ ] Navigation works (previous/next)
- [ ] "Waiting for input" indicator appears
- [ ] Resume button appears for waiting tasks
- [ ] Resume functionality works
- [ ] Task order consistent after resume

## Performance Tests

### Large Workflow

**Setup**: 100 tasks in sequential order

**Test**:
```rust
#[tokio::test]
async fn test_performance_large_workflow() {
    let workflow = create_large_workflow(100);
    let start = std::time::Instant::now();
    
    let result = execute_workflow_by_id(&workflow.id, None).await.unwrap();
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(5));
}
```

### Deeply Nested Workflow

**Setup**: 10 levels deep nesting

**Test**:
```rust
#[tokio::test]
async fn test_performance_deep_nesting() {
    let workflow = create_deeply_nested_workflow(10);
    
    // Test extraction performance
    let start = std::time::Instant::now();
    let task_ids = extract_task_ids(&workflow.content);
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_millis(100));
}
```

## Test Coverage

### Required Coverage

- **Model tests**: 100% coverage for WorkflowExecution
- **Store tests**: 100% coverage for execution store operations
- **API tests**: 100% coverage for execute_workflow_by_id
- **Integration tests**: All user input scenarios

### Test Files

1. `persistence/tests/models/execution_tests.rs`
2. `persistence/tests/store/execution_tests.rs`
3. `persistence/tests/integration_tests.rs`
4. `persistence/tests/concurrency_tests.rs`
5. `core/tests/api/execution_tests.rs`
6. `core/tests/integration_tests.rs`
7. `engine/tests/engine_tests.rs`

## Test Data

### Example Workflows

**File**: `engine/examples/user_input_simple.json`

```json
{
  "id": "user_input_simple",
  "name": "Simple User Input Workflow",
  "tasks": [
    {
      "id": "get_name",
      "name": "Get User Name",
      "function": {
        "user_input": {
          "prompt": "Enter your name:",
          "input_type": "string",
          "required": true
        }
      },
      "next_tasks": [
        {
          "id": "greet",
          "name": "Greet User",
          "function": {
            "cli_command": {
              "command": "echo",
              "args": ["Hello"]
            }
          },
          "next_tasks": []
        }
      ]
    }
  ]
}
```

## Continuous Integration

### GitHub Actions

```yaml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run tests
        run: cargo test
      - name: Run quality
        run: task quality
```

## Test Execution

### Run All Tests

```bash
cargo test
```

### Run Specific Tests

```bash
cargo test test_execution_with_snapshot
```

### Run with Logging

```bash
RUST_LOG=trace cargo test -- --nocapture
```

## Success Criteria

✅ All unit tests pass  
✅ All integration tests pass  
✅ Manual testing successful  
✅ Performance acceptable  
✅ Code coverage requirements met  
✅ Ready for production  

