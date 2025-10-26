# Phase 1: Persistence Layer - Add workflow_snapshot Field

## Objective

Add `workflow_snapshot: serde_json::Value` field to `WorkflowExecution` model to store complete workflow JSON structure.

## Status: ⏳ Pending

## Files to Modify

### 1. persistence/src/models/execution.rs

Add field to struct and update Default implementation:

```rust
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub workflow_snapshot: serde_json::Value,  // NEW
    pub status: WorkflowStatus,
    // ... existing fields
}

impl Default for WorkflowExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_name: String::new(),
            workflow_snapshot: serde_json::json!({}),  // NEW
            // ... rest of fields
        }
    }
}
```

### 2. persistence/tests/execution_tests.rs

Update `create_test_execution()` helper:

```rust
fn create_test_execution() -> WorkflowExecution {
    WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test Workflow",
            "tasks": []
        }),
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: HashMap::new(),
        errors: Vec::new(),
    }
}
```

### 3. persistence/tests/store/execution_tests.rs

Same update to `create_test_execution()` helper.

### 4. persistence/tests/models/execution_tests.rs

Update serialization test and add snapshot test:

```rust
#[test]
fn test_workflow_execution_serialization_with_snapshot() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test",
            "tasks": []
        }),
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: HashMap::new(),
        errors: Vec::new(),
    };
    
    let json = serde_json::to_string(&execution).unwrap();
    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.workflow_snapshot, execution.workflow_snapshot);
}
```

### 5. persistence/tests/integration_tests.rs

Update execution creation in tests.

### 6. persistence/tests/concurrency_tests.rs

Update execution creation in tests.

## Implementation Steps

1. ✅ Open `persistence/src/models/execution.rs`
2. ⏳ Add `workflow_snapshot: serde_json::Value` to struct
3. ⏳ Update Default implementation
4. ⏳ Update all test helpers with snapshot field
5. ⏳ Add serialization test
6. ⏳ Run tests: `cargo test -p persistence`

## Testing

### Run Tests

```bash
cd persistence
cargo test
```

### Expected Output

```
running 45 tests
test models::execution::test_workflow_execution_serialization_with_snapshot ... ok
test store::execution::test_save_workflow_execution ... ok
...
test result: ok. 45 passed; 0 failed
```

## Validation Checklist

- [ ] Code compiles without errors
- [ ] All existing tests still pass
- [ ] New snapshot field serializes correctly
- [ ] Model default includes empty snapshot
- [ ] All test helpers updated

## Rollback Plan

If issues arise:

```bash
git checkout persistence/src/models/execution.rs
git checkout persistence/tests/
```

## SRP Compliance

- ✅ Models file only contains model definitions
- ✅ Store files only contain CRUD operations
- ✅ Test files are separate and organized
- ✅ Each file has one responsibility

## Next Phase

After completion, proceed to **Phase 2: Core Integration**.

