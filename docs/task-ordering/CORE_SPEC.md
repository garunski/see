# Task Ordering - Core Layer Specification

## Overview

This specification describes the core layer changes required to store and preserve workflow snapshots during execution.

## Core API Changes

### execute_workflow_by_id

**File**: `core/src/api/execution.rs`

Store workflow snapshot when creating initial execution:

```rust
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    // ... existing load workflow code ...

    // Create Initial WorkflowExecution Record
    let execution_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // Parse workflow content to JSON
    let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)
        .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;

    let initial_execution = WorkflowExecution {
        id: execution_id.clone(),
        workflow_name: workflow.name.clone(),
        workflow_snapshot: workflow_json,  // NEW: Store snapshot
        status: WorkflowStatus::Running,
        created_at: now,
        completed_at: None,
        success: None,
        tasks: Vec::new(),
        timestamp: now,
        audit_trail: Vec::new(),
        per_task_logs: std::collections::HashMap::new(),
        errors: Vec::new(),
    };

    // ... rest of execution logic ...
}
```

### Preserve Snapshot on Update

When updating execution status for waiting tasks:

```rust
if has_input_waiting {
    tracing::info!("Workflow paused - waiting for user input");

    let mut updated_execution = initial_execution.clone();
    updated_execution.status = WorkflowStatus::Running;
    // workflow_snapshot automatically preserved via Clone

    store
        .save_workflow_execution(updated_execution)
        .await
        .map_err(CoreError::Persistence)?;
    
    // ... rest of pause logic ...
}
```

### Update Bridge Conversion

**File**: `core/src/bridge/execution.rs`

Add workflow_snapshot to bridge conversion (using empty JSON):

```rust
pub fn workflow_result_to_execution(
    result: EngineWorkflowResult,
    execution_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
) -> WorkflowExecution {
    let now = chrono::Utc::now();

    let task_executions = result
        .tasks
        .iter()
        .map(|task| {
            crate::bridge::task::task_info_to_execution(
                task,
                &execution_id,
                &result.per_task_logs,
                &result.errors,
                created_at,
                now,
            )
        })
        .collect();

    WorkflowExecution {
        id: execution_id,
        workflow_name: result.workflow_name,
        workflow_snapshot: serde_json::json!({}),  // Empty for engine conversions
        status: if result.success {
            WorkflowStatus::Complete
        } else {
            WorkflowStatus::Failed
        },
        created_at,
        completed_at: Some(now),
        success: Some(result.success),
        tasks: task_executions,
        timestamp: now,
        audit_trail: Vec::new(),
        per_task_logs: result.per_task_logs,
        errors: result.errors,
    }
}
```

**Note**: Engine conversions use empty JSON because engine doesn't have workflow context. Actual execution in `execute_workflow_by_id` uses real workflow JSON.

### Resume Task Enhancement

**File**: `core/src/api/resume.rs`

Preserve snapshot when updating execution:

```rust
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    // ... existing load execution code ...

    if all_tasks_complete {
        let mut updated_execution = execution.clone();
        updated_execution.status = WorkflowStatus::Complete;
        updated_execution.completed_at = Some(chrono::Utc::now());
        // workflow_snapshot automatically preserved via Clone

        store
            .save_workflow_execution(updated_execution)
            .await
            .map_err(CoreError::Persistence)?;
    }

    // ... rest of resume logic ...
}
```

## Error Handling

### Invalid JSON Handling

When parsing workflow content, handle errors gracefully:

```rust
let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)
    .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;
```

**Behavior**: Return CoreError::Execution if JSON is invalid. This prevents storing execution with invalid snapshot.

### Empty Snapshot Handling

If workflow content is empty (edge case):

```rust
let workflow_json = if workflow.content.is_empty() {
    serde_json::json!({})
} else {
    serde_json::from_str(&workflow.content)
        .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?
};
```

## Integration Points

### Workflow Definition Loading

**File**: `core/src/api/execution.rs`

Snapshot is populated from loaded WorkflowDefinition:

```rust
let workflow = store
    .get_workflow(workflow_id)
    .await
    .map_err(CoreError::Persistence)?
    .ok_or_else(|| CoreError::WorkflowNotFound(workflow_id.to_string()))?;

// workflow.content contains the complete JSON
let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)?;
```

### Engine Workflow Conversion

**File**: `core/src/bridge/workflow.rs`

No changes needed. Engine workflow conversion is separate from snapshot storage:

```rust
pub fn workflow_definition_to_engine(
    workflow: &WorkflowDefinition,
) -> Result<EngineWorkflow, CoreError> {
    // This converts workflow.content to EngineWorkflow
    // The snapshot is the original workflow.content
    let parsed = engine::parse_workflow(&workflow.content)
        .map_err(|e| CoreError::Engine(engine::EngineError::Parser(e)))?;
    Ok(parsed)
}
```

Snapshot = original JSON
Engine workflow = parsed structure

## SRP Compliance

### File Organization

- **api/execution.rs** - ONLY execution API
- **api/resume.rs** - ONLY resume API
- **bridge/execution.rs** - ONLY execution conversions
- **tests/api/** - ONLY API tests
- **tests/bridge/** - ONLY bridge tests

Each file has ONE responsibility.

## Test Updates

### Execution Tests

**File**: `core/tests/api/execution_tests.rs`

Update test data to include workflow_snapshot:

```rust
#[tokio::test]
async fn test_execute_workflow_stores_snapshot() {
    // Setup: Create workflow with content
    let workflow = WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        // ... other fields
    };
    store.save_workflow(&workflow).await.unwrap();

    // Execute
    let result = execute_workflow_by_id("test-workflow", None).await.unwrap();

    // Verify snapshot stored
    let execution = store.get_workflow_execution(&result.execution_id).await.unwrap().unwrap();
    
    assert!(!execution.workflow_snapshot.is_null());
    assert_eq!(execution.workflow_snapshot["id"], "test");
}
```

### Bridge Tests

**File**: `core/tests/bridge/execution_tests.rs`

Update bridge conversion tests:

```rust
#[test]
fn test_workflow_result_to_execution_includes_snapshot() {
    let result = EngineWorkflowResult {
        // ... result data
    };
    
    let execution = workflow_result_to_execution(result, "exec-1".to_string(), Utc::now());
    
    assert_eq!(execution.workflow_snapshot, serde_json::json!({}));
}
```

## Validation

### Requirements Checklist

✅ Store workflow_snapshot in execute_workflow_by_id
✅ Parse workflow.content to JSON
✅ Handle invalid JSON gracefully
✅ Preserve snapshot on execution update
✅ Preserve snapshot in resume_task
✅ Update bridge conversion function
✅ Add workflow_snapshot to engine conversions (empty JSON)
✅ Update all test files
✅ SRP compliance maintained

## Next Steps

After completing core layer changes:

1. Update GUI layer to use snapshot for ordering
2. Implement task ordering logic
3. Test end-to-end execution with ordering
4. Run integration tests
5. Proceed to bug investigation phase

