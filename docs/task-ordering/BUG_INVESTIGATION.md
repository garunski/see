# Workflow Pause Bug Investigation

## Problem Statement

**Critical Bug**: Workflows do not pause when encountering user input tasks. Instead, they continue execution and complete all tasks without waiting for user input.

## Expected Behavior

1. Engine encounters user input task
2. Task marked as `WaitingForInput`
3. Engine pauses execution loop
4. Core API detects paused state
5. Execution saved as paused
6. GUI displays "waiting for input" indicator
7. User provides input via resume API
8. Workflow continues from paused task

## Actual Behavior

1. Engine encounters user input task
2. Task executes
3. Workflow continues immediately
4. All remaining tasks execute
5. Workflow completes successfully
6. No pause mechanism activates

## Investigation Hypothesis

The pause mechanism has multiple checkpoints that should stop execution. We suspect one or more of these checkpoints is failing:

1. **UserInputHandler return value** - Returns `waiting_for_input: true` ✅
2. **Engine result processing** - Detects waiting_for_input ❓
3. **Loop break condition** - Should pause when waiting_for_input populated ❓
4. **Core API detection** - Checks TaskStatus::WaitingForInput ❓
5. **Status conversion** - Maps engine status to persistence status ❓

## Investigation Methodology

### Step 1: Enable Comprehensive Logging

Set maximum logging level to capture all engine activity:

```bash
export RUST_LOG="trace"
```

Or in code:
```rust
tracing_subscriber::fmt()
    .with_max_level(Level::TRACE)
    .init();
```

### Step 2: Instrument Critical Functions

Add detailed tracing to:

1. **engine/src/handlers/user_input.rs**
   - Before/after handler execution
   - Return value inspection
   - TaskResult construction

2. **engine/src/engine.rs**
   - Before/after execute_round
   - Result processing loop
   - waiting_for_input set updates
   - Loop break conditions

3. **core/src/api/execution.rs**
   - Engine result inspection
   - TaskStatus checking
   - Pause detection logic

### Step 3: Run Test Workflow

Execute a user input workflow and capture all logs:

```bash
# Use example workflow with user input
cd /Users/garunnvagidov/code/see
RUST_LOG=trace cargo run --bin see_cli workflow run user_input_simple.json
```

### Step 4: Analyze Log Output

Look for these key indicators:

```
✅ CORRECT BEHAVIOR:
1. "Starting user input task execution"
2. "Marking task as WaitingForInput"
3. "Task waiting for user input"
4. "workflow_snapshot: waiting_for_input: true"
5. "Workflow paused - waiting for X input(s)"
6. "WorkflowStatus: Running (paused)"

❌ BUG INDICATORS:
1. "Starting user input task execution"
2. "Task status: complete" (should be WaitingForInput)
3. "No waiting detected" (when there should be)
4. Workflow completes immediately
```

### Step 5: Identify Root Cause

Compare actual logs vs expected behavior to find failure point:

**File**: `engine/src/engine.rs` lines 210-222

Current code:
```rust
for (task, result) in results {
    // Check if task is waiting for input
    if let Some(waiting) = result.output.get("waiting_for_input") {
        if waiting.as_bool().unwrap_or(false) {
            waiting_for_input.insert(task.id.clone());
            debug!("Task waiting for user input");
            continue;
        }
    }
    
    if result.success {
        completed_tasks.insert(task.id.clone());
    }
}
```

**Potential Issues**:
1. `result.output` doesn't contain `waiting_for_input` key
2. `waiting.as_bool()` returns false
3. `waiting_for_input.insert()` doesn't affect loop
4. Loop continues instead of breaking

**File**: `core/src/api/execution.rs` lines 73-105

Current code:
```rust
let has_input_waiting = engine_result
    .tasks
    .iter()
    .any(|t| matches!(t.status, engine::TaskStatus::WaitingForInput));
```

**Potential Issues**:
1. `engine_result.tasks` doesn't contain WaitingForInput tasks
2. Status not properly converted from TaskResult
3. Check happens on wrong data structure

### Step 6: Implement Fix

Based on root cause analysis:

**If Issue**: Engine not detecting waiting_for_input
**Fix**: Add more logging, verify TaskResult output structure

**If Issue**: Status conversion failing
**Fix**: Update bridge conversion logic

**If Issue**: Core API not checking correctly
**Fix**: Update detection logic, add logging

**If Issue**: Loop not breaking
**Fix**: Review break condition, add explicit pause signal

### Step 7: Validate Fix

After implementing fix:

1. Re-run test workflow
2. Verify workflow pauses
3. Check logs show correct pause mechanism
4. Test resume functionality
5. Run all user input integration tests

## Key Files for Investigation

### 1. engine/src/engine.rs

**Lines 133-224**: Main execution loop
- `waiting_for_input` set management
- Result processing
- Loop break conditions

**Check**: Does the loop pause when `waiting_for_input` is populated?

### 2. engine/src/handlers/user_input.rs

**Lines 79-89**: Return value construction
- TaskResult with `waiting_for_input: true`
- TaskStatus set to WaitingForInput

**Check**: Is the return value correct?

### 3. engine/src/types.rs

**TaskStatus enum**: Defines WaitingForInput variant
**TaskResult struct**: Contains output JSON

**Check**: Status and result structures are correct?

### 4. core/src/api/execution.rs

**Lines 73-105**: Pause detection after engine execution
- Checks if any tasks have WaitingForInput status
- Returns early if detected

**Check**: Is the detection logic correct?

### 5. core/src/bridge/task.rs

**Status conversions**: Maps engine::TaskStatus to persistence::TaskStatus

**Check**: WaitingForInput maps correctly?

## Logging Strategy

Add detailed logs at critical points:

### In UserInputHandler (engine/src/handlers/user_input.rs)

```rust
debug!(
    execution_id = %context.execution_id,
    task_id = %task.id,
    "🔍 Handler returning waiting state"
);
trace!(
    output = ?serde_json::to_string(&result.output),
    "Return value structure"
);
```

### In Engine Loop (engine/src/engine.rs)

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
        // ...
    }
}
```

### In Core API (core/src/api/execution.rs)

```rust
debug!(
    tasks = %engine_result.tasks.len(),
    task_statuses = ?engine_result.tasks.iter().map(|t| &t.status).collect::<Vec<_>>(),
    "🔍 Inspecting engine result"
);

let has_input_waiting = engine_result
    .tasks
    .iter()
    .any(|t| matches!(t.status, engine::TaskStatus::WaitingForInput));

debug!(
    has_input_waiting,
    "🔍 Pause detection result"
);
```

## Test Cases

Create test cases to verify pause behavior:

```rust
#[tokio::test]
async fn test_workflow_pauses_for_user_input() {
    // 1. Create workflow with user input task
    // 2. Execute workflow
    // 3. Verify execution pauses
    // 4. Check task status is WaitingForInput
    // 5. Verify loop breaks
}

#[tokio::test]
async fn test_core_detects_paused_state() {
    // 1. Execute user input workflow
    // 2. Verify Core API detects waiting
    // 3. Check execution saved as paused
    // 4. Verify early return
}
```

## Success Criteria

Investigation is complete when:

✅ Logs show pause mechanism activating
✅ Workflow execution pauses at user input task
✅ TaskStatus is WaitingForInput
✅ Core API detects and handles pause
✅ GUI shows "waiting for input" indicator
✅ Resume functionality works correctly
✅ All integration tests pass

## Timeline

**Estimated Duration**: 2-4 hours

1. Enable logging (15 min)
2. Run test workflow (15 min)
3. Analyze logs (1 hour)
4. Identify root cause (30 min)
5. Implement fix (1 hour)
6. Test and validate (1 hour)

## Next Steps

After completing investigation:

1. Document root cause in this file
2. Implement fix with comprehensive logging
3. Add tests for pause/resume behavior
4. Update bug investigation status in phase docs
5. Proceed with snapshot implementation

