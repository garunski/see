# Phase 2: Basic Execution Context Pause/Resume (No Persistence)

## Overview

Add basic pause/resume functionality to the ExecutionContext. This phase focuses on in-memory state management only - no database persistence yet. The goal is to establish the core pause/resume mechanism that will be used by task handlers.

## Changes Required

### 1. Add Pause/Resume Methods to ExecutionContext

**File**: `core/src/execution/context.rs`

Add the following methods to the `ExecutionContext` implementation:

```rust
impl ExecutionContext {
    // ... existing methods ...

    /// Pause a task for user input
    pub fn pause_for_input(&mut self, task_id: &str, prompt: &str) -> Result<(), CoreError> {
        // Validate task exists
        if !self.tasks.iter().any(|t| t.id == task_id) {
            return Err(CoreError::Validation(format!("Task {} not found", task_id)));
        }

        // Log the pause
        self.log(&format!("⏸️  Task {} paused for user input: {}", task_id, prompt));
        
        // Update task status
        self.update_task_status(task_id, TaskStatus::WaitingForInput);
        
        // Set current task to None since it's paused
        self.current_task_id = None;
        
        Ok(())
    }

    /// Resume a paused task
    pub fn resume_task(&mut self, task_id: &str) -> Result<(), CoreError> {
        // Validate task exists and is waiting for input
        let task = self.tasks.iter().find(|t| t.id == task_id)
            .ok_or_else(|| CoreError::Validation(format!("Task {} not found", task_id)))?;
        
        if task.status != TaskStatus::WaitingForInput {
            return Err(CoreError::Validation(format!(
                "Task {} is not waiting for input (status: {})", 
                task_id, 
                task.status
            )));
        }

        // Log the resume
        self.log(&format!("▶️  Task {} resumed from user input pause", task_id));
        
        // Update task status back to InProgress
        self.update_task_status(task_id, TaskStatus::InProgress);
        
        // Set as current task
        self.current_task_id = Some(task_id.to_string());
        
        Ok(())
    }

    /// Check if any task is waiting for input
    pub fn has_waiting_tasks(&self) -> bool {
        self.tasks.iter().any(|t| t.status == TaskStatus::WaitingForInput)
    }

    /// Get all tasks waiting for input
    pub fn get_waiting_tasks(&self) -> Vec<&TaskInfo> {
        self.tasks.iter().filter(|t| t.status == TaskStatus::WaitingForInput).collect()
    }
}
```

### 2. Add Safe Wrapper Methods

**File**: `core/src/execution/context.rs`

Add safe wrapper methods to the `ExecutionContextSafe` trait:

```rust
impl ExecutionContextSafe for Arc<Mutex<ExecutionContext>> {
    // ... existing methods ...

    fn safe_pause_for_input(&self, task_id: &str, prompt: &str) -> Result<(), CoreError> {
        self.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?
            .pause_for_input(task_id, prompt)
    }

    fn safe_resume_task(&self, task_id: &str) -> Result<(), CoreError> {
        self.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?
            .resume_task(task_id)
    }

    fn safe_has_waiting_tasks(&self) -> Result<bool, CoreError> {
        Ok(self.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?
            .has_waiting_tasks())
    }

    fn safe_get_waiting_tasks(&self) -> Result<Vec<TaskInfo>, CoreError> {
        Ok(self.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?
            .get_waiting_tasks()
            .iter()
            .map(|t| (*t).clone())
            .collect())
    }
}
```

### 3. Update ExecutionContextSafe Trait

**File**: `core/src/execution/context.rs`

Add the new methods to the trait definition:

```rust
pub trait ExecutionContextSafe: Send + Sync {
    // ... existing methods ...

    fn safe_pause_for_input(&self, task_id: &str, prompt: &str) -> Result<(), CoreError>;
    fn safe_resume_task(&self, task_id: &str) -> Result<(), CoreError>;
    fn safe_has_waiting_tasks(&self) -> Result<bool, CoreError>;
    fn safe_get_waiting_tasks(&self) -> Result<Vec<TaskInfo>, CoreError>;
}
```

## Testing Phase 2

### Unit Test

Create a simple unit test to verify the pause/resume functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskInfo, TaskStatus};

    #[test]
    fn test_pause_resume_task() {
        // Create test context
        let tasks = vec![
            TaskInfo {
                id: "task1".to_string(),
                name: "Test Task".to_string(),
                status: TaskStatus::InProgress,
            }
        ];
        
        let context = ExecutionContext::new(
            tasks,
            None, // no output callback
            None, // no audit store
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        // Test pause
        {
            let mut ctx = context.lock().unwrap();
            ctx.start_task("task1");
            let result = ctx.pause_for_input("task1", "Continue?");
            assert!(result.is_ok());
            assert_eq!(ctx.tasks[0].status, TaskStatus::WaitingForInput);
            assert!(ctx.has_waiting_tasks());
            assert_eq!(ctx.get_waiting_tasks().len(), 1);
        }

        // Test resume
        {
            let mut ctx = context.lock().unwrap();
            let result = ctx.resume_task("task1");
            assert!(result.is_ok());
            assert_eq!(ctx.tasks[0].status, TaskStatus::InProgress);
            assert!(!ctx.has_waiting_tasks());
        }
    }

    #[test]
    fn test_pause_nonexistent_task() {
        let tasks = vec![];
        let context = ExecutionContext::new(
            tasks,
            None,
            None,
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        let mut ctx = context.lock().unwrap();
        let result = ctx.pause_for_input("nonexistent", "Continue?");
        assert!(result.is_err());
    }

    #[test]
    fn test_resume_non_waiting_task() {
        let tasks = vec![
            TaskInfo {
                id: "task1".to_string(),
                name: "Test Task".to_string(),
                status: TaskStatus::Complete,
            }
        ];
        
        let context = ExecutionContext::new(
            tasks,
            None,
            None,
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        let mut ctx = context.lock().unwrap();
        let result = ctx.resume_task("task1");
        assert!(result.is_err());
    }
}
```

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
```

**Expected Result**: Compiles without errors

### Integration Test
1. **Create simple test workflow** that uses the new methods
2. **Verify pause/resume works** in actual execution
3. **Check logs** for pause/resume messages

## Potential Failures

### 1. Mutex Lock Errors
**Error**: `Failed to lock context` errors
**Solution**: Ensure proper error handling in safe wrapper methods

### 2. Task Validation Errors
**Error**: Task not found or wrong status
**Solution**: Add proper validation in pause/resume methods

### 3. Import Errors
**Error**: `TaskStatus::WaitingForInput` not found
**Solution**: Ensure Phase 1 was completed successfully

### 4. Trait Implementation Errors
**Error**: Missing trait method implementations
**Solution**: Ensure all trait methods are implemented

### 5. Ownership Issues
**Error**: Move semantics violations
**Solution**: Use references where possible, clone when necessary

## Files Modified

- `core/src/execution/context.rs` - Main changes

## Success Criteria

### ✅ Compilation Success
- Core crate compiles without errors
- All trait methods implemented
- No type mismatches

### ✅ Functionality Works
- Tasks can be paused for input
- Tasks can be resumed from pause
- Status transitions work correctly
- Logging works as expected

### ✅ Error Handling
- Invalid task IDs are handled gracefully
- Wrong task statuses are handled gracefully
- Mutex lock failures are handled gracefully

### ✅ No Side Effects
- Existing functionality unchanged
- No performance regression
- No breaking changes

## Verification Checklist

- [ ] ExecutionContext has pause_for_input method
- [ ] ExecutionContext has resume_task method
- [ ] ExecutionContext has has_waiting_tasks method
- [ ] ExecutionContext has get_waiting_tasks method
- [ ] ExecutionContextSafe trait has safe wrapper methods
- [ ] All methods implemented correctly
- [ ] Unit tests pass
- [ ] Core crate compiles without errors
- [ ] No functional changes to existing code

## Implementation Details

### Pause Logic
1. **Validate task exists** - Return error if not found
2. **Log pause message** - Include task ID and prompt
3. **Update task status** - Set to WaitingForInput
4. **Clear current task** - Set current_task_id to None

### Resume Logic
1. **Validate task exists** - Return error if not found
2. **Validate task status** - Must be WaitingForInput
3. **Log resume message** - Include task ID
4. **Update task status** - Set back to InProgress
5. **Set current task** - Set current_task_id to task_id

### Error Handling
- **Task not found**: Return Validation error
- **Wrong status**: Return Validation error
- **Mutex lock failure**: Return MutexLock error
- **All errors logged** for debugging

## Next Steps

After Phase 2 is complete and verified:
1. Proceed to Phase 3 (GUI Status Indicators)
2. Ensure all success criteria are met
3. Do not proceed if there are any compilation errors

## Notes

- This phase focuses on in-memory state only
- No database persistence yet
- The pause/resume mechanism is the foundation for future phases
- Test thoroughly with unit tests before proceeding
- This phase should have ZERO functional impact on existing code
