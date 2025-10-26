# Phase 2: Core Integration - Store Snapshot on Execution

## Objective

Store workflow snapshot when creating executions and preserve it in updates.

## Status: ⏳ Pending

## Files to Modify

### 1. core/src/api/execution.rs

Update `execute_workflow_by_id` to parse and store snapshot:

```rust
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    // ... existing load workflow code ...

    // Parse workflow content to JSON for snapshot
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

    // ... rest of function unchanged ...
}
```

No changes needed for the pause detection logic - snapshot preserved via clone.

### 2. core/src/bridge/execution.rs

Update `workflow_result_to_execution` to include snapshot field:

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
        workflow_snapshot: serde_json::json!({}),  // Empty for conversions
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

### 3. core/src/api/resume.rs

Already preserves via `.clone()` - no changes needed.

### 4. Update Tests

**File**: `core/tests/api/execution_tests.rs`

Add snapshot assertion:

```rust
#[tokio::test]
async fn test_execute_workflow_stores_snapshot() {
    let workflow = WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    store.save_workflow(&workflow).await.unwrap();
    
    let result = execute_workflow_by_id("test-workflow", None).await.unwrap();
    
    let execution = store.get_workflow_execution(&result.execution_id)
        .await
        .unwrap()
        .unwrap();
    
    assert!(!execution.workflow_snapshot.is_null());
    assert_eq!(execution.workflow_snapshot["id"], "test");
}
```

## Implementation Steps

1. ⏳ Update `core/src/api/execution.rs` line 45-57
2. ⏳ Update `core/src/bridge/execution.rs` line 30-46
3. ⏳ Update test files
4. ⏳ Run tests: `cargo test -p s_e_e_core`

## Testing

### Run Tests

```bash
cd core
cargo test
```

### Expected Output

```
running 25 tests
test api::execution::test_execute_workflow_stores_snapshot ... ok
test bridge::execution::test_conversion ... ok
...
test result: ok. 25 passed; 0 failed
```

## Validation Checklist

- [ ] Code compiles without errors
- [ ] Executions include workflow_snapshot when created
- [ ] Snapshot preserved on updates
- [ ] Resume preserves snapshot
- [ ] All tests pass

## SRP Compliance

- ✅ API files only contain API logic
- ✅ Bridge files only contain conversions
- ✅ Test files are separate and organized
- ✅ Each file has one responsibility

## Next Phase

After completion, proceed to **Phase 3: GUI Ordering**.

