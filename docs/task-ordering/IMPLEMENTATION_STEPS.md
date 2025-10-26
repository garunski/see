# Task Ordering - Implementation Steps

## Phase Overview

| Phase | Focus | Duration | Dependencies |
|-------|-------|---------|--------------|
| Phase 1 | Persistence Layer | 1 hour | None |
| Phase 2 | Core Integration | 1.5 hours | Phase 1 |
| Phase 3 | GUI Ordering | 1.5 hours | Phase 1, 2 |
| Phase 4 | Bug Investigation | 2-4 hours | None |
| Phase 5 | Testing | 2 hours | Phases 1-4 |
| Phase 6 | Documentation | 1 hour | All phases |

## Phase 1: Persistence Layer

### Objective

Add `workflow_snapshot: serde_json::Value` field to `WorkflowExecution` model.

### Steps

#### 1.1 Update Model

**File**: `persistence/src/models/execution.rs`

Add field to struct:
```rust
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub workflow_snapshot: serde_json::Value,  // NEW
    // ... existing fields
}
```

Update Default implementation:
```rust
impl Default for WorkflowExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            // ... existing fields
            workflow_snapshot: serde_json::json!({}),  // NEW
            // ... rest of fields
        }
    }
}
```

#### 1.2 Update Test Helpers

**Files to update**:
1. `persistence/tests/execution_tests.rs`
2. `persistence/tests/store/execution_tests.rs`
3. `persistence/tests/models/execution_tests.rs`
4. `persistence/tests/integration_tests.rs`
5. `persistence/tests/concurrency_tests.rs`

Add to all test execution creations:
```rust
workflow_snapshot: serde_json::json!({
    "id": "test",
    "name": "Test Workflow",
    "tasks": []
}),
```

#### 1.3 Add Serialization Test

**File**: `persistence/tests/models/execution_tests.rs`

```rust
#[test]
fn test_workflow_execution_serialization_with_snapshot() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test",
            "tasks": []
        }),
        // ... other fields
    };
    
    let json = serde_json::to_string(&execution).unwrap();
    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.workflow_snapshot, execution.workflow_snapshot);
}
```

### Validation

- [ ] Compiles without errors
- [ ] All tests pass
- [ ] Database operations work
- [ ] Serialization/deserialization works

### Rollback

If issues arise:
```bash
git checkout persistence/src/models/execution.rs
git checkout persistence/tests/
```

## Phase 2: Core Integration

### Objective

Store workflow snapshot when creating executions and preserve it in updates.

### Steps

#### 2.1 Update execute_workflow_by_id

**File**: `core/src/api/execution.rs` (lines 45-57)

Add snapshot parsing:
```rust
let workflow_json: serde_json::Value = serde_json::from_str(&workflow.content)
    .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;

let initial_execution = WorkflowExecution {
    id: execution_id.clone(),
    workflow_name: workflow.name.clone(),
    workflow_snapshot: workflow_json,  // NEW
    status: WorkflowStatus::Running,
    // ... rest of fields
};
```

#### 2.2 Update Bridge Conversion

**File**: `core/src/bridge/execution.rs` (lines 30-46)

Add to return:
```rust
WorkflowExecution {
    id: execution_id,
    workflow_name: result.workflow_name,
    workflow_snapshot: serde_json::json!({}),  // Empty for conversions
    // ... rest of fields
}
```

#### 2.3 Preserve in Resume

**File**: `core/src/api/resume.rs`

Already preserved via `.clone()` - no changes needed.

#### 2.4 Update Tests

**Files**: `core/tests/api/execution_tests.rs`, `core/tests/bridge/execution_tests.rs`

Add snapshot assertions to all test executions.

### Validation

- [ ] Compiles without errors
- [ ] Executions include workflow_snapshot
- [ ] Snapshot preserved on updates
- [ ] Resume preserves snapshot
- [ ] All tests pass

## Phase 3: GUI Ordering

### Objective

Implement task ordering logic in GUI using workflow snapshot.

### Steps

#### 3.1 Create Hook

**File**: `gui/src/pages/executions/details/hooks.rs`

Add functions:
```rust
pub fn use_task_order_from_snapshot(
    execution: Signal<Option<WorkflowExecution>>,
) -> Memo<Vec<String>> {
    use_memo(move || {
        execution()
            .map(|exec| extract_task_ids_recursive(&exec.workflow_snapshot))
            .unwrap_or_default()
    })
}

fn extract_task_ids_recursive(value: &serde_json::Value) -> Vec<String> {
    // ... implementation from GUI_SPEC.md
}
```

#### 3.2 Update Details Page

**File**: `gui/src/pages/executions/details/page.rs`

Add hook call and task reordering logic.

### Validation

- [ ] Compiles without errors
- [ ] Tasks display in correct order
- [ ] Navigation works
- [ ] Panel works with reordered tasks

## Phase 4: Bug Investigation

### Objective

Debug why workflows don't pause for user input.

### Steps

#### 4.1 Enable Logging

```bash
export RUST_LOG=trace
```

#### 4.2 Run Test Workflow

```bash
cd /Users/garunnvagidov/code/see
cargo run --bin see_cli workflow run engine/examples/user_input_simple.json
```

#### 4.3 Analyze Logs

Look for:
- UserInputHandler execution
- Task status changes
- waiting_for_input set updates
- Loop break conditions

#### 4.4 Identify Root Cause

Compare logs vs expected behavior to find failure point.

#### 4.5 Implement Fix

Patch the issue based on investigation.

#### 4.6 Validate Fix

- [ ] Workflow pauses correctly
- [ ] GUI shows "waiting for input"
- [ ] Resume works
- [ ] Tests pass

### Rollback

Document root cause and fix approach in BUG_INVESTIGATION.md.

## Phase 5: Testing

### Objective

Comprehensive testing of task ordering and pause functionality.

### Steps

#### 5.1 Unit Tests

Run all unit tests:
```bash
cargo test
```

#### 5.2 Integration Tests

Test end-to-end:
```bash
cargo test --test integration
```

#### 5.3 Manual Testing

1. Create workflow with user input
2. Execute workflow
3. Verify pause
4. Provide input
5. Verify resume
6. Check task order in GUI

#### 5.4 Performance Tests

- Test with large workflows (50+ tasks)
- Test with deeply nested workflows
- Measure reordering performance

### Validation

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Manual testing successful
- [ ] Performance acceptable

## Phase 6: Documentation and Quality

### Objective

Final documentation and code quality checks.

### Steps

#### 6.1 Update README

Add documentation about:
- Task ordering feature
- Workflow snapshot
- User input pause behavior

#### 6.2 Run Quality Checks

```bash
task quality
```

This runs:
- `cargo clippy`
- `cargo fmt --check`
- `cargo test`

#### 6.3 Final Review

- [ ] All documentation complete
- [ ] Code passes quality checks
- [ ] All tests pass
- [ ] SRP compliance verified
- [ ] Ready for production

## Common Issues and Solutions

### Issue: Compilation Errors

**Problem**: Missing workflow_snapshot field in tests
**Solution**: Update all test helpers to include field

### Issue: Empty Snapshots

**Problem**: Snapshot is `{}` (empty JSON)
**Solution**: Ensure parse_workflow happens before execution

### Issue: Order Still Wrong

**Problem**: Tasks still display in wrong order
**Solution**: Check extract_task_ids_recursive logic

### Issue: Workflow Doesn't Pause

**Problem**: Still completing instead of pausing
**Solution**: Follow Phase 4 investigation

## Success Criteria

✅ All phases complete
✅ All tests pass
✅ Task ordering works correctly
✅ Workflow pause works correctly
✅ GUI displays tasks in order
✅ Resume functionality works
✅ `task quality` passes
✅ Documentation complete

