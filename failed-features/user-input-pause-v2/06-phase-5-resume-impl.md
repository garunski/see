# Phase 5: Actual Resume Implementation (Core Engine)

## Overview

Implement the actual resume functionality in the core engine. This phase connects the GUI resume button to the core pause/resume logic implemented in Phase 2. The goal is to make the resume button actually work by updating task status in the database.

## Changes Required

### 1. Add Resume Workflow Function

**File**: `core/src/engine/execute.rs`

Add a new function to resume a paused workflow:

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
    
    // Update workflow status back to Running
    let mut updated_metadata = metadata.clone();
    updated_metadata.status = crate::persistence::models::WorkflowStatus::Running;
    store.save_workflow_metadata(&updated_metadata).await?;
    
    info!(
        execution_id = %execution_id,
        "Workflow resumed successfully"
    );
    
    Ok(())
}
```

### 2. Add Resume Task Function

**File**: `core/src/engine/execute.rs`

Add a function to resume a specific task:

```rust
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
        // Update workflow status back to Running
        let metadata = store.get_workflow_metadata(execution_id).await?;
        let mut updated_metadata = metadata.clone();
        updated_metadata.status = crate::persistence::models::WorkflowStatus::Running;
        store.save_workflow_metadata(&updated_metadata).await?;
        
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

### 3. Update GUI Resume Button

**File**: `gui/src/pages/executions/details/page.rs`

Update the resume button to call the actual resume function:

```rust
// Update the resume button onclick handler
button { 
    class: "px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors",
    onclick: move |_| {
        let execution_id_clone = execution_id.clone();
        let task_id_clone = task.id.clone();
        
        spawn(async move {
            tracing::info!("Resume button clicked for execution {} task {}", execution_id_clone, task_id_clone);
            
            match s_e_e_core::engine::resume_task(&execution_id_clone, &task_id_clone).await {
                Ok(_) => {
                    tracing::info!("Task resumed successfully");
                    // TODO: Refresh the page or update state in Phase 6
                }
                Err(e) => {
                    tracing::error!("Failed to resume task: {}", e);
                    // TODO: Show error message to user in Phase 6
                }
            }
        });
    },
    "Resume Workflow"
}
```

### 4. Update Task Details Panel Resume Button

**File**: `gui/src/pages/executions/details/components/task_details_panel.rs`

Update the resume button in the task details panel:

```rust
// Update the resume button onclick handler
button { 
    class: "px-3 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded text-sm font-medium transition-colors",
    onclick: move |_| {
        let task_id_clone = task.id.clone();
        let execution_id_clone = execution_id.clone();
        
        spawn(async move {
            tracing::info!("Resume button clicked for task {}", task_id_clone);
            
            match s_e_e_core::engine::resume_task(&execution_id_clone, &task_id_clone).await {
                Ok(_) => {
                    tracing::info!("Task resumed successfully");
                    // TODO: Refresh the page or update state in Phase 6
                }
                Err(e) => {
                    tracing::error!("Failed to resume task: {}", e);
                    // TODO: Show error message to user in Phase 6
                }
            }
        });
    },
    "Resume Task"
}
```

## Testing Phase 5

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Functionality Test
1. **Create test data** with WaitingForInput status in database
2. **Navigate to execution details** page
3. **Click resume button**
4. **Check database** to verify task status updated
5. **Refresh page** to verify status changed

### Test Data Creation
Use the same test data from previous phases:

```sql
-- Insert a test workflow with waiting status
INSERT INTO workflow_metadata (id, workflow_name, start_timestamp, end_timestamp, status, task_ids)
VALUES ('test-waiting', 'Test Waiting Workflow', '2024-12-19T10:00:00Z', NULL, 'waiting-for-input', '["task1"]');

-- Insert a test task with waiting status  
INSERT INTO task_executions (execution_id, task_id, task_name, status, logs, start_timestamp, end_timestamp)
VALUES ('test-waiting', 'task1', 'Test Task', 'waiting-for-input', '["Task paused for input"]', '2024-12-19T10:00:00Z', '');
```

### Database Verification
After clicking resume, check the database:

```sql
-- Check task status
SELECT task_id, status, end_timestamp FROM task_executions WHERE execution_id = 'test-waiting';

-- Check workflow status
SELECT status FROM workflow_metadata WHERE id = 'test-waiting';
```

**Expected Result**: 
- Task status should be `in-progress`
- Workflow status should be `running`
- End timestamp should be empty

## Potential Failures

### 1. Database Connection Errors
**Error**: `Failed to get global store` or database connection errors
**Solution**: Ensure database is running and accessible

### 2. Task Not Found
**Error**: `Task not found in execution`
**Solution**: Verify test data is correct and task exists

### 3. Wrong Task Status
**Error**: `Task is not waiting for input`
**Solution**: Ensure task status is `waiting-for-input` in test data

### 4. Move Semantics Violations
**Error**: `use of moved value` in GUI code
**Solution**: Clone variables before using in closures

### 5. Async Spawn Issues
**Error**: `future cannot be sent between threads safely`
**Solution**: Use `spawn` instead of `tokio::spawn` in GUI code

## Files Modified

- `core/src/engine/execute.rs` - Add resume functions
- `gui/src/pages/executions/details/page.rs` - Update resume button
- `gui/src/pages/executions/details/components/task_details_panel.rs` - Update resume button

## Success Criteria

### ✅ Compilation Success
- Both crates compile without errors
- No type mismatches
- All imports resolved correctly

### ✅ Resume Functionality Works
- Resume button actually resumes tasks
- Task status updated in database
- Workflow status updated correctly
- Logging works as expected

### ✅ Error Handling
- Invalid execution IDs handled gracefully
- Invalid task IDs handled gracefully
- Wrong task statuses handled gracefully
- Database errors handled gracefully

### ✅ No Side Effects
- Existing functionality unchanged
- No performance regression
- No breaking changes

## Verification Checklist

- [ ] resume_workflow function implemented
- [ ] resume_task function implemented
- [ ] GUI resume buttons call resume functions
- [ ] Task status updated in database
- [ ] Workflow status updated correctly
- [ ] Error handling works
- [ ] Both crates compile without errors
- [ ] No functional changes to existing code

## Implementation Details

### Resume Workflow Logic
1. **Load workflow metadata** - Verify it exists and is waiting
2. **Load task executions** - Find all waiting tasks
3. **Update task statuses** - Set to InProgress
4. **Update workflow status** - Set to Running
5. **Log all operations** - For debugging

### Resume Task Logic
1. **Load task execution** - Verify it exists and is waiting
2. **Update task status** - Set to InProgress
3. **Check other tasks** - See if any are still waiting
4. **Update workflow status** - Only if no tasks waiting
5. **Log all operations** - For debugging

### Error Handling
- **Workflow not found**: Return Validation error
- **Task not found**: Return Validation error
- **Wrong status**: Return Validation error
- **Database errors**: Propagate with context
- **All errors logged** for debugging

## Next Steps

After Phase 5 is complete and verified:
1. Proceed to Phase 6 (Simple Persistence)
2. Ensure all success criteria are met
3. Do not proceed if there are any compilation errors

## Notes

- This phase implements the core resume functionality
- Database operations are performed asynchronously
- Error handling is comprehensive
- Logging is extensive for debugging
- GUI buttons now actually work
- Test thoroughly with database verification
