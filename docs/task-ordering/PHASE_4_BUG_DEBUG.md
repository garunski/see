# Phase 4: Bug Investigation and Fix - Workflow Pause Issue

## Objective

Debug and fix the critical bug where workflows don't pause for user input.

## Status: ⏳ Pending

## Problem

Workflows complete execution instead of pausing when encountering user input tasks.

## Investigation Steps

### Step 1: Enable Logging

Set maximum log level:

```bash
export RUST_LOG=trace
```

Or in Rust code:
```rust
tracing_subscriber::fmt()
    .with_max_level(Level::TRACE)
    .init();
```

### Step 2: Add Instrumentation

**File**: `engine/src/handlers/user_input.rs`

Add detailed logging:

```rust
impl TaskHandler for UserInputHandler {
    async fn execute(&self, context: &mut ExecutionContext, task: &EngineTask) -> Result<TaskResult, HandlerError> {
        debug!(execution_id = %context.execution_id, task_id = %task.id, "🔍 Starting user input task");
        
        // ... existing handler logic ...

        let result = TaskResult {
            success: true,
            output: serde_json::json!({
                "waiting_for_input": true,
                "prompt": prompt,
                "input_type": input_type,
                "required": required,
                "default": default.clone(),
            }),
            error: None,
        };

        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            output = ?serde_json::to_string(&result.output),
            "🔍 Handler returning waiting state"
        );

        Ok(result)
    }
}
```

**File**: `engine/src/engine.rs` (lines 210-222)

Add detailed logging:

```rust
for (task, result) in results {
    trace!(
        task_id = %task.id,
        success = result.success,
        output_keys = ?result.output.as_object().map(|o| o.keys().collect::<Vec<_>>()),
        "Processing task result"
    );
    
    if let Some(waiting) = result.output.get("waiting_for_input") {
        debug!(
            task_id = %task.id,
            waiting = ?waiting,
            "🔍 Checking waiting_for_input flag"
        );
        
        if waiting.as_bool().unwrap_or(false) {
            waiting_for_input.insert(task.id.clone());
            debug!(task_id = %task.id, "Task added to waiting_for_input set");
            continue;
        }
    }
    
    // ... rest of processing
}

debug!(
    execution_id = %execution_id,
    waiting_count = waiting_for_input.len(),
    "Execution loop complete"
);
```

**File**: `core/src/api/execution.rs` (lines 73-105)

Add detailed logging:

```rust
let has_input_waiting = engine_result
    .tasks
    .iter()
    .any(|t| matches!(t.status, engine::TaskStatus::WaitingForInput));

debug!(
    task_count = engine_result.tasks.len(),
    task_statuses = ?engine_result.tasks.iter().map(|t| &t.status).collect::<Vec<_>>(),
    has_input_waiting,
    "🔍 Inspecting engine result"
);
```

### Step 3: Run Test Workflow

Execute a user input workflow:

```bash
cd /Users/garunnvagidov/code/see
RUST_LOG=trace cargo run --bin see_cli workflow run engine/examples/user_input_simple.json
```

### Step 4: Analyze Logs

Look for these patterns:

**✅ CORRECT BEHAVIOR**:
```
🔍 Starting user input task
🔍 Handler returning waiting state {"waiting_for_input": true, ...}
Task added to waiting_for_input set
Execution loop complete: waiting_count = 1
🔍 Inspecting engine result: has_input_waiting = true
Workflow paused - waiting for user input
```

**❌ BUG INDICATORS**:
```
🔍 Starting user input task
Handler returning result (missing waiting flag)
All tasks complete (when should be waiting)
has_input_waiting = false (when should be true)
Workflow completes immediately
```

### Step 5: Identify Root Cause

Based on log analysis:

**Potential Issue 1**: UserInputHandler not returning correct output
**Location**: `engine/src/handlers/user_input.rs` lines 79-89
**Fix**: Ensure output contains "waiting_for_input": true

**Potential Issue 2**: Engine not detecting waiting_for_input
**Location**: `engine/src/engine.rs` lines 211-222
**Fix**: Verify result.output.get("waiting_for_input") check

**Potential Issue 3**: Loop not breaking
**Location**: `engine/src/engine.rs` lines 163-182
**Fix**: Verify break condition when waiting_for_input populated

**Potential Issue 4**: Core API not detecting pause
**Location**: `core/src/api/execution.rs` line 76
**Fix**: Verify TaskStatus::WaitingForInput checking

**Potential Issue 5**: Status conversion issue
**Location**: `core/src/bridge/task.rs`
**Fix**: Verify engine::TaskStatus → persistence::TaskStatus mapping

### Step 6: Implement Fix

Based on investigation results, implement fix.

### Step 7: Validate Fix

After implementing fix:

1. Re-run test workflow
2. Verify workflow pauses correctly
3. Check logs show correct pause mechanism
4. Test resume functionality
5. Run integration tests

## Expected Outcomes

### Success Indicators

✅ Workflow pauses at user input task
✅ Task status is WaitingForInput
✅ Core API detects pause
✅ Execution saved as paused
✅ GUI shows "waiting for input" indicator
✅ Resume button appears
✅ Resume functionality works

### Validation Tests

```rust
#[tokio::test]
async fn test_workflow_pauses_for_user_input() {
    let workflow = create_user_input_workflow();
    let result = engine.execute_workflow(workflow).await.unwrap();
    
    assert!(result.tasks.iter().any(|t| t.status == TaskStatus::WaitingForInput));
    assert!(!result.success);
}
```

## Timeline

**Estimated Duration**: 2-4 hours

- Enable logging: 15 min
- Run test workflow: 15 min
- Analyze logs: 1 hour
- Identify root cause: 30 min
- Implement fix: 1 hour
- Test and validate: 1 hour

## Documentation

Document findings in this file as investigation progresses:

```markdown
## Investigation Results

### Root Cause Identified
[Description of root cause]

### Fix Applied
[Description of fix]

### Verification
[Test results confirming fix works]
```

## Next Phase

After completion, proceed to **Phase 5: Testing**.

