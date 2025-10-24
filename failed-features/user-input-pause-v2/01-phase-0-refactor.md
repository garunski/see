# Phase 0: MANDATORY - Refactor Task Persistence (Eliminate Code Duplication)

## Overview

**CRITICAL**: This phase MUST be completed before ANY work on user input pause feature begins. The previous implementation would have tripled the duplicated code by adding user input logic to 6 duplicate code blocks.

## Current Problem

Both `CliCommandHandler` and `CursorAgentHandler` contain **identical** code blocks (~50 lines each, 3 times per handler):

1. **Task start persistence** (lines 113-137 in cli_command.rs, 179-203 in cursor_agent.rs)
2. **Task failure persistence** (lines 176-200 in cli_command.rs, 237-261 in cursor_agent.rs)  
3. **Task completion persistence** (lines 241-265 in cli_command.rs, 303-327 in cursor_agent.rs)

**Total duplication**: ~150 lines of identical code

## Architecture Solution

Create a shared `TaskPersistenceHelper` that both handlers use, eliminating all duplication.

### Implementation

#### Step 1: Create TaskPersistenceHelper

**File**: `core/src/task_executor.rs`

Add the following struct and implementation:

```rust
use crate::types::TaskStatus;
use std::sync::{Arc, Mutex};
use tracing::{error, trace, Instrument};

pub struct TaskPersistenceHelper {
    context: Arc<Mutex<crate::execution::context::ExecutionContext>>,
}

impl TaskPersistenceHelper {
    pub fn new(context: Arc<Mutex<crate::execution::context::ExecutionContext>>) -> Self {
        Self { context }
    }
    
    /// Save task state (start/failed/complete) to database asynchronously
    pub fn save_task_state_async(&self, task_id: &str, status: TaskStatus) {
        let Ok(ctx) = self.context.lock() else {
            error!("Failed to lock context for task persistence");
            return;
        };
        
        let Some(store) = ctx.get_store() else {
            return; // No store configured
        };
        
        let task_exec = crate::persistence::models::TaskExecution {
            execution_id: ctx.get_execution_id(),
            task_id: task_id.to_string(),
            task_name: task_id.to_string(),
            status,
            logs: ctx.get_task_logs(task_id),
            start_timestamp: ctx.get_task_start_time(task_id),
            end_timestamp: if status == TaskStatus::InProgress {
                String::new()
            } else {
                chrono::Utc::now().to_rfc3339()
            },
        };
        drop(ctx);
        
        let status_str = status.as_str();
        let span = tracing::debug_span!("save_task_state_bg", task_id = %task_id, status = %status_str);
        tokio::spawn(
            async move {
                trace!("Saving task state to database");
                if let Err(e) = store.save_task_execution(&task_exec).await {
                    error!(error = %e, "Failed to save task state");
                }
            }
            .instrument(span),
        );
    }
}
```

#### Step 2: Update CliCommandHandler

**File**: `core/src/engine/handlers/cli_command.rs`

1. **Add import**:
```rust
use crate::task_executor::TaskPersistenceHelper;
```

2. **Add field to struct**:
```rust
pub struct CliCommandHandler {
    context: Arc<Mutex<ExecutionContext>>,
    persistence: TaskPersistenceHelper,  // Add this line
}
```

3. **Update constructor**:
```rust
impl CliCommandHandler {
    pub fn new(context: Arc<Mutex<ExecutionContext>>) -> Self {
        Self { 
            context: context.clone(),
            persistence: TaskPersistenceHelper::new(context),
        }
    }
}
```

4. **Replace task start persistence** (lines 113-137):
```rust
// REPLACE THIS BLOCK:
if let Ok(ctx) = self.context.lock() {
    if let Some(store) = ctx.get_store() {
        let task_exec = crate::persistence::models::TaskExecution {
            execution_id: ctx.get_execution_id(),
            task_id: task_id.to_string(),
            task_name: task_id.to_string(),
            status: TaskStatus::InProgress,
            logs: ctx.get_task_logs(task_id),
            start_timestamp: ctx.get_task_start_time(task_id),
            end_timestamp: String::new(),
        };
        drop(ctx);

        let span = tracing::debug_span!("save_task_start_bg", task_id = %task_id);
        tokio::spawn(
            async move {
                trace!("Saving task start state");
                if let Err(e) = store.save_task_execution(&task_exec).await {
                    error!(error = %e, "Failed to save task start");
                }
            }
            .instrument(span),
        );
    }
}

// WITH THIS SINGLE LINE:
self.persistence.save_task_state_async(task_id, TaskStatus::InProgress);
```

5. **Replace task failure persistence** (lines 176-200):
```rust
// REPLACE THE ENTIRE BLOCK WITH:
self.persistence.save_task_state_async(task_id, TaskStatus::Failed);
```

6. **Replace task completion persistence** (lines 241-265):
```rust
// REPLACE THE ENTIRE BLOCK WITH:
self.persistence.save_task_state_async(task_id, TaskStatus::Complete);
```

#### Step 3: Update CursorAgentHandler

**File**: `core/src/engine/handlers/cursor_agent.rs`

Apply the same changes as CliCommandHandler:

1. **Add import**:
```rust
use crate::task_executor::TaskPersistenceHelper;
```

2. **Add field to struct**:
```rust
pub struct CursorAgentHandler {
    context: Arc<Mutex<ExecutionContext>>,
    persistence: TaskPersistenceHelper,  // Add this line
}
```

3. **Update constructor**:
```rust
impl CursorAgentHandler {
    pub fn new(context: Arc<Mutex<ExecutionContext>>) -> Self {
        Self { 
            context: context.clone(),
            persistence: TaskPersistenceHelper::new(context),
        }
    }
}
```

4. **Replace all 3 persistence blocks** with single helper calls:
```rust
// Task start (lines 179-203):
self.persistence.save_task_state_async(task_id, TaskStatus::InProgress);

// Task failure (lines 237-261):
self.persistence.save_task_state_async(task_id, TaskStatus::Failed);

// Task completion (lines 303-327):
self.persistence.save_task_state_async(task_id, TaskStatus::Complete);
```

## Testing Phase 0

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
```

**Expected Result**: Compiles without errors

### Functionality Test
1. **Run existing workflow** to verify refactored persistence works
2. **Check logs** to confirm task states are saved
3. **Verify database** contains correct task execution records

**Expected Result**: Identical behavior to before refactoring

### Code Quality Test
1. **Count lines** in both handler files
2. **Verify duplication eliminated**
3. **Check that TaskPersistenceHelper is used** in both handlers

**Expected Result**: ~140 lines removed from each handler file

## Potential Failures

### 1. Import Errors
**Error**: `use crate::task_executor::TaskPersistenceHelper;` fails
**Solution**: Ensure TaskPersistenceHelper is in the same module as other task_executor items

### 2. Mutex Lock Failures
**Error**: Context lock fails in TaskPersistenceHelper
**Solution**: Add proper error handling (already included in implementation)

### 3. Handler Initialization
**Error**: Handlers fail to construct TaskPersistenceHelper
**Solution**: Ensure context is cloned properly in constructors

### 4. Span Recording
**Error**: Tracing spans don't work correctly
**Solution**: Verify `tracing` imports are correct

### 5. Store Access
**Error**: Store is None when trying to save
**Solution**: Early return is already handled in implementation

## Success Criteria

### ✅ Code Duplication Eliminated
- **Before**: ~150 lines duplicated across 2 handlers
- **After**: ~30 lines total in TaskPersistenceHelper
- **Reduction**: ~80% reduction in duplicated code

### ✅ Existing Functionality Preserved
- Workflows execute identically to before
- Task states saved to database correctly
- No performance regression
- All logging works as expected

### ✅ Clean Architecture
- Single point of truth for task persistence
- Easy to test persistence logic
- Future features can be added in one place
- No code duplication

### ✅ Compilation Success
- Core crate compiles without errors
- All imports resolved correctly
- No type mismatches

## Verification Checklist

- [ ] `cargo build -p s_e_e_core` succeeds
- [ ] TaskPersistenceHelper compiles without errors
- [ ] CliCommandHandler uses TaskPersistenceHelper
- [ ] CursorAgentHandler uses TaskPersistenceHelper
- [ ] All 6 duplicate blocks replaced with helper calls
- [ ] Existing workflow executes successfully
- [ ] Task states saved to database
- [ ] No performance regression
- [ ] Code duplication eliminated

## Why This Phase is Mandatory

**Without this refactoring**:
- Adding user input pause logic would require modifying 6 duplicate code blocks
- Each modification is an opportunity for bugs and inconsistencies
- Testing would require verifying 6 separate locations
- Future maintenance would be nightmare

**With this refactoring**:
- User input pause logic added in ONE place (TaskPersistenceHelper)
- Single point of testing and verification
- Future task state management improvements benefit all handlers
- Clean architecture for Phase 5 resume implementation

## Next Steps

After Phase 0 is complete and verified:
1. Proceed to Phase 1 (Core Type System Updates)
2. Do NOT skip Phase 0 - it is mandatory
3. Ensure all success criteria are met before continuing

## Notes

- This refactoring should have ZERO functional impact
- The behavior must be identical to before
- If any behavior changes, the refactoring is incorrect
- Take time to verify thoroughly - this is the foundation for all future work
