# Phase 4: Simple Resume Button (Manual Trigger, No Auto-Loading)

## Overview

Add a simple resume button that appears when a task is waiting for user input. This phase focuses on the UI interaction only - the button logs the action but doesn't implement actual resume logic yet. No use_effect hooks, no auto-loading, no state management.

## Changes Required

### 1. Add Resume Button to Execution Details Page

**File**: `gui/src/pages/executions/details/page.rs`

Add a resume button that appears when a task is in `WaitingForInput` status:

```rust
// Find the task details section and add the resume button
if let Some(task) = exec.tasks.get(selected_task_index()) {
    if task.status == s_e_e_core::TaskStatus::WaitingForInput {
        rsx! {
            div { 
                class: "bg-amber-50 border border-amber-200 rounded-lg p-4 mb-4",
                div { class: "flex items-center gap-2 mb-2",
                    span { class: "text-amber-600 text-lg", "⏸️" }
                    span { class: "text-amber-800 font-medium", "Task Waiting for Input" }
                }
                p { class: "text-amber-700 text-sm mb-3",
                    "This task is paused and waiting for user input. Click the button below to resume execution."
                }
                button { 
                    class: "px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors",
                    onclick: move |_| {
                        tracing::info!("Resume button clicked for execution {} task {}", execution_id, task.id);
                        // TODO: Implement actual resume logic in Phase 5
                    },
                    "Resume Workflow"
                }
            }
        }
    }
}
```

### 2. Add Resume Button to Task Details Panel

**File**: `gui/src/pages/executions/details/components/task_details_panel.rs`

Add a resume button in the task details panel when the task is waiting:

```rust
// Find the task status section and add the resume button
if task.status == s_e_e_core::TaskStatus::WaitingForInput {
    rsx! {
        div { class: "mt-4 p-3 bg-amber-50 border border-amber-200 rounded",
            div { class: "flex items-center gap-2 mb-2",
                span { class: "text-amber-600", "⏸️" }
                span { class: "text-amber-800 font-medium", "Waiting for Input" }
            }
            p { class: "text-amber-700 text-sm mb-3",
                "This task is paused and waiting for user input."
            }
            button { 
                class: "px-3 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded text-sm font-medium transition-colors",
                onclick: move |_| {
                    tracing::info!("Resume button clicked for task {}", task.id);
                    // TODO: Implement actual resume logic in Phase 5
                },
                "Resume Task"
            }
        }
    }
}
```

## Testing Phase 4

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Manual Testing
1. **Create test data** with WaitingForInput status in database
2. **Navigate to execution details** page
3. **Verify resume button appears** for waiting tasks
4. **Click resume button** and check logs
5. **Verify no infinite loops** or performance issues

### Test Data Creation
Use the same test data from Phase 3:

```sql
-- Insert a test workflow with waiting status
INSERT INTO workflow_metadata (id, workflow_name, start_timestamp, end_timestamp, status, task_ids)
VALUES ('test-waiting', 'Test Waiting Workflow', '2024-12-19T10:00:00Z', NULL, 'waiting-for-input', '["task1"]');

-- Insert a test task with waiting status  
INSERT INTO task_executions (execution_id, task_id, task_name, status, logs, start_timestamp, end_timestamp)
VALUES ('test-waiting', 'task1', 'Test Task', 'waiting-for-input', '["Task paused for input"]', '2024-12-19T10:00:00Z', '');
```

### Log Verification
After clicking the resume button, check the logs for:
```
INFO [gui]: Resume button clicked for execution test-waiting task task1
```

## Potential Failures

### 1. Move Semantics Violations
**Error**: `use of moved value: execution_id`
**Solution**: Clone variables before using in closures

```rust
// WRONG - This will cause move errors
let handle_resume = move |_| {
    tracing::info!("Resume button clicked for execution {}", execution_id);
};

// CORRECT - Clone before using in closure
let execution_id_clone = execution_id.clone();
let handle_resume = move |_| {
    tracing::info!("Resume button clicked for execution {}", execution_id_clone);
};
```

### 2. Missing Imports
**Error**: `tracing::info!` not found
**Solution**: Add tracing import at the top of the file

```rust
use tracing::info;
```

### 3. Component Props Issues
**Error**: Missing required props for components
**Solution**: Ensure all required props are passed to components

### 4. Status Comparison Errors
**Error**: Cannot compare TaskStatus with WaitingForInput
**Solution**: Ensure Phase 1 was completed successfully

## Files Modified

- `gui/src/pages/executions/details/page.rs`
- `gui/src/pages/executions/details/components/task_details_panel.rs`

## Success Criteria

### ✅ Compilation Success
- GUI crate compiles without errors
- No move semantics violations
- All imports resolved correctly

### ✅ UI Elements Work
- Resume button appears for waiting tasks
- Button click logs message correctly
- No visual glitches or layout issues
- Button styling is appropriate

### ✅ No Performance Issues
- No infinite loops
- No excessive re-renders
- No memory leaks
- Button click is responsive

### ✅ No Functional Changes
- Existing functionality unchanged
- No side effects
- No breaking changes

## Verification Checklist

- [ ] Resume button appears in execution details page for waiting tasks
- [ ] Resume button appears in task details panel for waiting tasks
- [ ] Button click logs message correctly
- [ ] No move semantics violations
- [ ] GUI crate compiles without errors
- [ ] No performance issues
- [ ] Button styling is appropriate

## Implementation Details

### Button Styling
- **Color**: Amber theme to match waiting status
- **Hover**: Darker amber on hover
- **Size**: Appropriate for the context
- **Spacing**: Proper margins and padding

### Button Text
- **Execution Details**: "Resume Workflow"
- **Task Details**: "Resume Task"
- **Consistent**: Clear, actionable text

### Button Behavior
- **Click**: Logs message and does nothing else
- **No Async**: No database calls or complex operations
- **No State**: No state management or effects

### Logging
- **Level**: INFO level for visibility
- **Content**: Include execution ID and task ID
- **Format**: Consistent with existing logging patterns

## Common Implementation Patterns

### Execution Details Page
```rust
// In page.rs
if let Some(task) = exec.tasks.get(selected_task_index()) {
    if task.status == s_e_e_core::TaskStatus::WaitingForInput {
        let execution_id_clone = execution_id.clone();
        let task_id_clone = task.id.clone();
        
        rsx! {
            div { class: "bg-amber-50 border border-amber-200 rounded-lg p-4 mb-4",
                // ... status display ...
                button { 
                    class: "px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors",
                    onclick: move |_| {
                        tracing::info!("Resume button clicked for execution {} task {}", execution_id_clone, task_id_clone);
                    },
                    "Resume Workflow"
                }
            }
        }
    }
}
```

### Task Details Panel
```rust
// In task_details_panel.rs
if task.status == s_e_e_core::TaskStatus::WaitingForInput {
    let task_id_clone = task.id.clone();
    
    rsx! {
        div { class: "mt-4 p-3 bg-amber-50 border border-amber-200 rounded",
            // ... status display ...
            button { 
                class: "px-3 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded text-sm font-medium transition-colors",
                onclick: move |_| {
                    tracing::info!("Resume button clicked for task {}", task_id_clone);
                },
                "Resume Task"
            }
        }
    }
}
```

## Next Steps

After Phase 4 is complete and verified:
1. Proceed to Phase 5 (Actual Resume Implementation)
2. Ensure all success criteria are met
3. Do not proceed if there are any compilation errors

## Notes

- This phase is purely UI - no functionality yet
- No use_effect hooks or state management
- No database queries or async operations
- Focus on clean, responsive UI
- Test thoroughly with manual test data
- Button should be clearly visible and accessible
