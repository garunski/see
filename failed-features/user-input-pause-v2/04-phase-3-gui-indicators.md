# Phase 3: GUI Status Indicators (Read-Only, No State Management)

## Overview

Add GUI indicators to show when tasks are waiting for user input. This phase is read-only - no state management, no use_effect hooks, no database queries. The goal is to display the waiting status clearly in the execution details screen.

## Changes Required

### 1. Update Task Details Panel

**File**: `gui/src/pages/executions/details/components/task_details_panel.rs`

Add handling for the `WaitingForInput` status:

```rust
// Find the status display section and add the new case
match task.status {
    s_e_e_core::TaskStatus::Pending => "⏳ Pending",
    s_e_e_core::TaskStatus::InProgress => "🔄 In Progress", 
    s_e_e_core::TaskStatus::Complete => "✅ Complete",
    s_e_e_core::TaskStatus::Failed => "❌ Failed",
    s_e_e_core::TaskStatus::WaitingForInput => "⏸️ Waiting for Input",  // Add this line
}
```

### 2. Update Execution Overview

**File**: `gui/src/pages/executions/details/components/execution_overview.rs`

Add waiting status badge:

```rust
// Find the status badge section and add the new case
match execution.status {
    s_e_e_core::persistence::models::WorkflowStatus::Running => "🔄 Running",
    s_e_e_core::persistence::models::WorkflowStatus::Complete => "✅ Complete",
    s_e_e_core::persistence::models::WorkflowStatus::Failed => "❌ Failed",
    s_e_e_core::persistence::models::WorkflowStatus::WaitingForInput => "⏸️ Waiting for Input",  // Add this line
}
```

### 3. Update Workflow Flow

**File**: `gui/src/pages/executions/details/components/workflow_flow.rs`

Add color coding for waiting tasks:

```rust
// Find the color mapping section and add the new case
fn get_task_color(task: &s_e_e_core::types::TaskInfo) -> &'static str {
    match task.status {
        s_e_e_core::TaskStatus::Complete => "#10b981",      // green
        s_e_e_core::TaskStatus::Failed => "#ef4444",        // red
        s_e_e_core::TaskStatus::InProgress => "#3b82f6",    // blue
        s_e_e_core::TaskStatus::Pending => "#6b7280",       // gray
        s_e_e_core::TaskStatus::WaitingForInput => "#f59e0b",  // amber - Add this line
    }
}
```

## Testing Phase 3

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
3. **Verify status indicators** show correctly
4. **Check colors** are appropriate (amber for waiting)

### Test Data Creation
To test the GUI indicators, you can manually insert test data into the database:

```sql
-- Insert a test workflow with waiting status
INSERT INTO workflow_metadata (id, workflow_name, start_timestamp, end_timestamp, status, task_ids)
VALUES ('test-waiting', 'Test Waiting Workflow', '2024-12-19T10:00:00Z', NULL, 'waiting-for-input', '["task1"]');

-- Insert a test task with waiting status  
INSERT INTO task_executions (execution_id, task_id, task_name, status, logs, start_timestamp, end_timestamp)
VALUES ('test-waiting', 'task1', 'Test Task', 'waiting-for-input', '["Task paused for input"]', '2024-12-19T10:00:00Z', '');
```

## Potential Failures

### 1. Non-Exhaustive Pattern Matching
**Error**: `error[E0004]: non-exhaustive patterns: TaskStatus::WaitingForInput not covered`
**Solution**: Ensure all match statements include the new variant

### 2. Import Errors
**Error**: `s_e_e_core::TaskStatus` not found
**Solution**: Add proper imports at the top of files

### 3. Type Mismatches
**Error**: Expected `WorkflowStatus` but found `TaskStatus`
**Solution**: Use the correct enum type for each context

### 4. Color Format Issues
**Error**: Invalid color format
**Solution**: Use valid hex color codes

## Files Modified

- `gui/src/pages/executions/details/components/task_details_panel.rs`
- `gui/src/pages/executions/details/components/execution_overview.rs`
- `gui/src/pages/executions/details/components/workflow_flow.rs`

## Success Criteria

### ✅ Compilation Success
- GUI crate compiles without errors
- No non-exhaustive pattern matching errors
- All imports resolved correctly

### ✅ Visual Indicators Work
- Waiting status displays with correct text
- Waiting status shows with correct color (amber)
- Status badges appear correctly
- No visual glitches or layout issues

### ✅ No Functional Changes
- Existing statuses display identically
- No performance impact
- No behavior changes

## Verification Checklist

- [ ] Task details panel shows "⏸️ Waiting for Input" for WaitingForInput status
- [ ] Execution overview shows "⏸️ Waiting for Input" for WaitingForInput status
- [ ] Workflow flow shows amber color (#f59e0b) for WaitingForInput status
- [ ] All match statements are exhaustive
- [ ] GUI crate compiles without errors
- [ ] No visual regressions for existing statuses

## Implementation Details

### Status Text
- **Task Details**: "⏸️ Waiting for Input"
- **Execution Overview**: "⏸️ Waiting for Input"
- **Consistent**: Use same text across all components

### Color Scheme
- **WaitingForInput**: Amber (#f59e0b)
- **Rationale**: Amber indicates caution/waiting, distinct from other statuses
- **Accessibility**: High contrast with white background

### Icons
- **WaitingForInput**: ⏸️ (pause symbol)
- **Consistent**: Use same icon across all components
- **Unicode**: Standard pause symbol, widely supported

## Common Pattern Examples

### Task Status Display
```rust
// In task_details_panel.rs
rsx! {
    div { class: "flex items-center gap-2",
        span { class: "text-sm font-medium",
            match task.status {
                s_e_e_core::TaskStatus::Pending => "⏳ Pending",
                s_e_e_core::TaskStatus::InProgress => "🔄 In Progress",
                s_e_e_core::TaskStatus::Complete => "✅ Complete",
                s_e_e_core::TaskStatus::Failed => "❌ Failed",
                s_e_e_core::TaskStatus::WaitingForInput => "⏸️ Waiting for Input",
            }
        }
    }
}
```

### Status Badge
```rust
// In execution_overview.rs
rsx! {
    span { 
        class: match execution.status {
            s_e_e_core::persistence::models::WorkflowStatus::Running => "px-2 py-1 bg-blue-100 text-blue-800 rounded",
            s_e_e_core::persistence::models::WorkflowStatus::Complete => "px-2 py-1 bg-green-100 text-green-800 rounded",
            s_e_e_core::persistence::models::WorkflowStatus::Failed => "px-2 py-1 bg-red-100 text-red-800 rounded",
            s_e_e_core::persistence::models::WorkflowStatus::WaitingForInput => "px-2 py-1 bg-amber-100 text-amber-800 rounded",
        },
        match execution.status {
            s_e_e_core::persistence::models::WorkflowStatus::Running => "🔄 Running",
            s_e_e_core::persistence::models::WorkflowStatus::Complete => "✅ Complete",
            s_e_e_core::persistence::models::WorkflowStatus::Failed => "❌ Failed",
            s_e_e_core::persistence::models::WorkflowStatus::WaitingForInput => "⏸️ Waiting for Input",
        }
    }
}
```

### Color Mapping
```rust
// In workflow_flow.rs
fn get_task_color(task: &s_e_e_core::types::TaskInfo) -> &'static str {
    match task.status {
        s_e_e_core::TaskStatus::Complete => "#10b981",      // green
        s_e_e_core::TaskStatus::Failed => "#ef4444",        // red
        s_e_e_core::TaskStatus::InProgress => "#3b82f6",    // blue
        s_e_e_core::TaskStatus::Pending => "#6b7280",       // gray
        s_e_e_core::TaskStatus::WaitingForInput => "#f59e0b",  // amber
    }
}
```

## Next Steps

After Phase 3 is complete and verified:
1. Proceed to Phase 4 (Simple Resume Button)
2. Ensure all success criteria are met
3. Do not proceed if there are any compilation errors

## Notes

- This phase is purely visual - no functionality changes
- No use_effect hooks or state management
- No database queries or async operations
- Focus on clear, consistent visual indicators
- Test with manually created test data
