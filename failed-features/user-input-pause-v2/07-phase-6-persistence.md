# Phase 6: Simple Persistence (Minimal Schema Changes)

## Overview

Add simple persistence for paused workflows so they survive app restarts. This phase adds minimal database schema changes and implements basic pause/resume persistence. The goal is to make paused workflows persistent across app restarts.

## Changes Required

### 1. Update WorkflowMetadata Schema

**File**: `core/src/persistence/models.rs`

Add pause-related fields to the `WorkflowMetadata` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub workflow_name: String,
    pub start_timestamp: String,
    pub end_timestamp: Option<String>,
    pub status: WorkflowStatus,
    pub task_ids: Vec<String>,
    // Add these fields
    pub is_paused: bool,
    pub paused_task_id: Option<String>,
}
```

### 2. Add Pause/Resume Methods to Store

**File**: `core/src/persistence/store.rs`

Add methods to mark workflows as paused and resumed:

```rust
impl RedbStore {
    /// Mark a workflow as paused for user input
    pub async fn mark_workflow_paused(&self, execution_id: &str, task_id: &str) -> Result<(), CoreError> {
        let mut metadata = self.get_workflow_metadata(execution_id).await?;
        
        metadata.is_paused = true;
        metadata.paused_task_id = Some(task_id.to_string());
        metadata.status = WorkflowStatus::WaitingForInput;
        
        self.save_workflow_metadata(&metadata).await?;
        
        info!(
            execution_id = %execution_id,
            task_id = %task_id,
            "Workflow marked as paused"
        );
        
        Ok(())
    }
    
    /// Mark a workflow as resumed from pause
    pub async fn mark_workflow_resumed(&self, execution_id: &str) -> Result<(), CoreError> {
        let mut metadata = self.get_workflow_metadata(execution_id).await?;
        
        metadata.is_paused = false;
        metadata.paused_task_id = None;
        metadata.status = WorkflowStatus::Running;
        
        self.save_workflow_metadata(&metadata).await?;
        
        info!(
            execution_id = %execution_id,
            "Workflow marked as resumed"
        );
        
        Ok(())
    }
    
    /// Get all paused workflows
    pub async fn get_paused_workflows(&self) -> Result<Vec<WorkflowMetadata>, CoreError> {
        let mut paused_workflows = Vec::new();
        
        // This is a simple implementation - in production you might want to add an index
        let all_workflows = self.list_workflow_executions(1000).await?;
        
        for workflow in all_workflows {
            if workflow.status == WorkflowStatus::WaitingForInput {
                paused_workflows.push(workflow);
            }
        }
        
        info!(
            count = paused_workflows.len(),
            "Found paused workflows"
        );
        
        Ok(paused_workflows)
    }
}
```

### 3. Update Resume Functions

**File**: `core/src/engine/execute.rs`

Update the resume functions to use the new persistence methods:

```rust
/// Resume a workflow that is waiting for user input
#[instrument(skip(execution_id), fields(execution_id))]
pub async fn resume_workflow(execution_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;
    
    info!(
        execution_id = %execution_id,
        "Resuming workflow execution"
    );
    
    // Load workflow metadata
    let metadata = store.get_workflow_metadata(execution_id).await?;
    
    // Check if workflow is actually waiting for input
    if metadata.status != crate::persistence::models::WorkflowStatus::WaitingForInput {
        return Err(CoreError::Validation(format!(
            "Workflow {} is not waiting for input (status: {})",
            execution_id,
            metadata.status
        )));
    }
    
    // Load task executions
    let task_executions = store.get_task_executions(execution_id).await?;
    
    // Find tasks that are waiting for input
    let waiting_tasks: Vec<_> = task_executions
        .iter()
        .filter(|task| task.status == crate::types::TaskStatus::WaitingForInput)
        .collect();
    
    if waiting_tasks.is_empty() {
        return Err(CoreError::Validation(format!(
            "No tasks waiting for input in workflow {}",
            execution_id
        )));
    }
    
    // Resume each waiting task
    for task in waiting_tasks {
        let mut updated_task = task.clone();
        updated_task.status = crate::types::TaskStatus::InProgress;
        updated_task.end_timestamp = String::new(); // Clear end timestamp
        
        store.save_task_execution(&updated_task).await?;
        
        info!(
            execution_id = %execution_id,
            task_id = %task.task_id,
            "Resumed task from waiting state"
        );
    }
    
    // Mark workflow as resumed using new method
    store.mark_workflow_resumed(execution_id).await?;
    
    info!(
        execution_id = %execution_id,
        "Workflow resumed successfully"
    );
    
    Ok(())
}

/// Resume a specific task that is waiting for user input
#[instrument(skip(execution_id, task_id), fields(execution_id, task_id))]
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;
    
    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Resuming specific task"
    );
    
    // Load task execution
    let task_executions = store.get_task_executions(execution_id).await?;
    let task = task_executions
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| CoreError::Validation(format!(
            "Task {} not found in execution {}",
            task_id,
            execution_id
        )))?;
    
    // Check if task is waiting for input
    if task.status != crate::types::TaskStatus::WaitingForInput {
        return Err(CoreError::Validation(format!(
            "Task {} is not waiting for input (status: {})",
            task_id,
            task.status
        )));
    }
    
    // Update task status
    let mut updated_task = task.clone();
    updated_task.status = crate::types::TaskStatus::InProgress;
    updated_task.end_timestamp = String::new(); // Clear end timestamp
    
    store.save_task_execution(&updated_task).await?;
    
    // Check if all tasks are now running or complete
    let all_tasks = store.get_task_executions(execution_id).await?;
    let has_waiting_tasks = all_tasks.iter().any(|t| t.status == crate::types::TaskStatus::WaitingForInput);
    
    if !has_waiting_tasks {
        // Mark workflow as resumed using new method
        store.mark_workflow_resumed(execution_id).await?;
        
        info!(
            execution_id = %execution_id,
            "All tasks resumed, workflow status updated to Running"
        );
    }
    
    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Task resumed successfully"
    );
    
    Ok(())
}
```

### 4. Add Pause Function

**File**: `core/src/engine/execute.rs`

Add a function to pause a workflow for user input:

```rust
/// Pause a workflow for user input
#[instrument(skip(execution_id, task_id), fields(execution_id, task_id))]
pub async fn pause_workflow(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    let store = crate::get_global_store()?;
    
    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Pausing workflow for user input"
    );
    
    // Load task execution
    let task_executions = store.get_task_executions(execution_id).await?;
    let task = task_executions
        .iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| CoreError::Validation(format!(
            "Task {} not found in execution {}",
            task_id,
            execution_id
        )))?;
    
    // Update task status to waiting for input
    let mut updated_task = task.clone();
    updated_task.status = crate::types::TaskStatus::WaitingForInput;
    updated_task.end_timestamp = String::new(); // Clear end timestamp
    
    store.save_task_execution(&updated_task).await?;
    
    // Mark workflow as paused
    store.mark_workflow_paused(execution_id, task_id).await?;
    
    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Workflow paused successfully"
    );
    
    Ok(())
}
```

## Testing Phase 6

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Database Migration Test
1. **Start the app** to trigger database migration
2. **Check database schema** to verify new fields added
3. **Verify existing data** is not corrupted

### Persistence Test
1. **Create test workflow** with waiting status
2. **Restart the app**
3. **Verify paused workflow** is still there
4. **Resume the workflow**
5. **Verify status** is updated correctly

### Test Data Creation
```sql
-- Insert a test workflow with waiting status
INSERT INTO workflow_metadata (id, workflow_name, start_timestamp, end_timestamp, status, task_ids, is_paused, paused_task_id)
VALUES ('test-waiting', 'Test Waiting Workflow', '2024-12-19T10:00:00Z', NULL, 'waiting-for-input', '["task1"]', true, 'task1');

-- Insert a test task with waiting status  
INSERT INTO task_executions (execution_id, task_id, task_name, status, logs, start_timestamp, end_timestamp)
VALUES ('test-waiting', 'task1', 'Test Task', 'waiting-for-input', '["Task paused for input"]', '2024-12-19T10:00:00Z', '');
```

### Database Verification
After restart, check the database:

```sql
-- Check workflow metadata
SELECT id, status, is_paused, paused_task_id FROM workflow_metadata WHERE id = 'test-waiting';

-- Check task status
SELECT task_id, status FROM task_executions WHERE execution_id = 'test-waiting';
```

**Expected Result**: 
- Workflow should have `is_paused = true` and `paused_task_id = 'task1'`
- Task should have status `waiting-for-input`

## Potential Failures

### 1. Database Migration Issues
**Error**: Database schema migration fails
**Solution**: Ensure backward compatibility with existing data

### 2. Serialization Errors
**Error**: New fields cause serialization/deserialization issues
**Solution**: Ensure new fields have default values

### 3. Backward Compatibility
**Error**: Existing workflows don't have new fields
**Solution**: Add default values for new fields

### 4. Database Connection Errors
**Error**: Database operations fail
**Solution**: Ensure database is running and accessible

## Files Modified

- `core/src/persistence/models.rs` - Add new fields to WorkflowMetadata
- `core/src/persistence/store.rs` - Add pause/resume methods
- `core/src/engine/execute.rs` - Update resume functions, add pause function

## Success Criteria

### ✅ Compilation Success
- Both crates compile without errors
- No serialization errors
- All imports resolved correctly

### ✅ Persistence Works
- Paused workflows survive app restarts
- Resume functionality works after restart
- Database schema updated correctly
- No data loss

### ✅ Backward Compatibility
- Existing workflows work unchanged
- No migration errors
- Default values work correctly

### ✅ No Side Effects
- Existing functionality unchanged
- No performance regression
- No breaking changes

## Verification Checklist

- [ ] WorkflowMetadata has new fields
- [ ] Store has pause/resume methods
- [ ] Resume functions use new methods
- [ ] Pause function implemented
- [ ] Database migration works
- [ ] Persistence works across restarts
- [ ] Both crates compile without errors
- [ ] No functional changes to existing code

## Implementation Details

### Database Schema Changes
- **is_paused**: Boolean flag for paused state
- **paused_task_id**: Which task is paused (if any)
- **Default values**: `false` and `None` for existing workflows

### Pause Logic
1. **Update task status** - Set to WaitingForInput
2. **Mark workflow paused** - Set is_paused = true
3. **Set paused task** - Set paused_task_id
4. **Update workflow status** - Set to WaitingForInput

### Resume Logic
1. **Update task status** - Set to InProgress
2. **Check other tasks** - See if any still waiting
3. **Mark workflow resumed** - Set is_paused = false
4. **Clear paused task** - Set paused_task_id = None
5. **Update workflow status** - Set to Running

### Error Handling
- **Workflow not found**: Return Validation error
- **Task not found**: Return Validation error
- **Database errors**: Propagate with context
- **All operations logged** for debugging

## Next Steps

After Phase 6 is complete and verified:
1. All phases are complete
2. User input pause feature is fully functional
3. Ready for production use

## Notes

- This phase completes the user input pause feature
- Persistence is simple but effective
- Backward compatibility is maintained
- Test thoroughly with app restarts
- Database migration should be seamless
