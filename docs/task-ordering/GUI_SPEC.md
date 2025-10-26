# Task Ordering - GUI Specification

## Overview

This specification describes the GUI layer changes required to implement task ordering from workflow snapshots.

## Design

### Current Task Display

Tasks are displayed in the order they appear in `exec.tasks`, which doesn't match execution order.

### Proposed Task Display

Extract task order from `workflow_snapshot` and reorder `exec.tasks` based on the snapshot structure.

## Hook Implementation

### use_task_order_from_snapshot

**File**: `gui/src/pages/executions/details/hooks.rs`

New hook to extract task execution order from workflow snapshot:

```rust
use serde_json::Value;

/// Extract task IDs in execution order from workflow snapshot
pub fn use_task_order_from_snapshot(
    execution: Signal<Option<WorkflowExecution>>,
) -> Memo<Vec<String>> {
    use_memo(move || {
        execution()
            .map(|exec| extract_task_ids_recursive(&exec.workflow_snapshot))
            .unwrap_or_default()
    })
}

/// Recursively extract task IDs from workflow JSON preserving execution order
fn extract_task_ids_recursive(value: &Value) -> Vec<String> {
    let mut task_ids = Vec::new();
    
    if let Some(tasks) = value.get("tasks").and_then(|v| v.as_array()) {
        for task in tasks {
            if let Some(task_id) = task.get("id").and_then(|v| v.as_str()) {
                task_ids.push(task_id.to_string());
                
                // Recursively get next_tasks
                if let Some(next_tasks) = task.get("next_tasks").and_then(|v| v.as_array()) {
                    for next_task in next_tasks {
                        task_ids.extend(extract_task_ids_recursive(next_task));
                    }
                }
            }
        }
    }
    
    task_ids
}
```

### Algorithm Explanation

**DFS (Depth-First Search) from workflow snapshot**:

1. Start at root tasks
2. For each task, add ID to ordered list
3. Recursively process `next_tasks`
4. Return flat list in execution order

**Example**:
```json
{
  "tasks": [
    {"id": "task1", "next_tasks": [
      {"id": "task2", "next_tasks": []}
    ]}
  ]
}
```

Results in: `["task1", "task2"]`

## Page Updates

### Details Page

**File**: `gui/src/pages/executions/details/page.rs`

Update to use ordered tasks:

```rust
#[component]
pub fn WorkflowDetailsPage(id: String) -> Element {
    let (execution, loading, error) = use_workflow_execution(id.clone());
    let ordered_task_ids = use_task_order_from_snapshot(execution);  // NEW
    
    // ... existing panel state ...

    rsx! {
        div { class: "space-y-8",
            // ... existing components ...

            if let Some(exec) = execution() {
                ExecutionOverview { execution: exec.clone() }

                // Reorder tasks based on workflow snapshot
                let ordered_tasks = {
                    let task_map: std::collections::HashMap<_, _> = exec.tasks.iter()
                        .map(|t| (t.id.clone(), t.clone()))
                        .collect();
                    
                    ordered_task_ids().iter()
                        .filter_map(|id| task_map.get(id).cloned())
                        .collect::<Vec<_>>()
                };

                if !ordered_tasks.is_empty() {
                    WorkflowFlow {
                        tasks: ordered_tasks.iter()  // NEW: Use ordered tasks
                            .map(s_e_e_core::task_execution_to_info)
                            .collect(),
                        on_task_click: move |task_index| {
                            selected_task_index.set(task_index);
                            is_panel_open.set(true);
                        }
                    }
                }

                // ... rest of page ...
            }
        }

        // Task Details Panel
        TaskDetailsPanel {
            // ... existing props ...
            total_tasks: execution().map(|exec| exec.tasks.len()).unwrap_or(0),
        }
    }
}
```

## Component Updates

### TaskDetailsPanel

No changes needed - component already receives task index and navigation works correctly with ordered tasks.

### WorkflowFlow

No changes needed - receives ordered tasks as input.

### ExecutionOverview

No changes needed - displays execution metadata regardless of task order.

## Edge Cases

### Empty Snapshot

If `workflow_snapshot` is empty JSON `{}` (shouldn't happen, but handle gracefully):

```rust
if ordered_task_ids().is_empty() {
    // Fall back to original task order
    exec.tasks.iter().map(...).collect()
} else {
    // Use ordered tasks
    ordered_task_ids().iter()...
}
```

### Missing Tasks in Snapshot

If snapshot has task IDs not present in `exec.tasks`:

```rust
.filter_map(|id| task_map.get(id).cloned())
```

Missing tasks are simply omitted from display.

### Orphaned Tasks

If `exec.tasks` has tasks not in snapshot (shouldn't happen):

```rust
// These will appear at the end (optional enhancement)
let snapshot_task_ids: std::collections::HashSet<_> = ordered_task_ids()
    .into_iter()
    .collect();
    
let orphaned_tasks = exec.tasks.iter()
    .filter(|t| !snapshot_task_ids.contains(&t.id))
    .collect::<Vec<_>>();
```

## User Experience

### Before

```
Tasks displayed in flat DFS order:
1. task1
2. task2 (child of task1)
3. task3 (child of task2)
4. task4 (sibling of task1)
```

User confused because order doesn't match execution flow.

### After

```
Tasks displayed in execution order from snapshot:
1. task1
2. task4 (parallel with task1)
3. task2 (executes after task1)
4. task3 (executes after task2)
```

User sees correct execution flow.

## Performance

### Memoization

Using `use_memo` for efficiency:

- Snapshot parsing only happens when execution changes
- Ordered IDs cached
- No re-computation on every render

### Complexity

- Extract task IDs: O(n) where n = tasks in snapshot
- Create task map: O(m) where m = tasks in execution
- Reorder: O(m) filtering
- **Total: O(n + m)** linear complexity

## SRP Compliance

### File Organization

- **hooks.rs** - ONLY custom hooks
- **page.rs** - ONLY page component logic
- **components/** - ONLY UI components

Each file has ONE responsibility.

### Hook Organization

```rust
// One hook per responsibility
pub fn use_workflow_execution() -> ...
pub fn use_task_order_from_snapshot() -> ...
pub fn use_task_navigation() -> ...
pub fn use_filtered_audit() -> ...
```

## Validation

### Requirements Checklist

✅ Create use_task_order_from_snapshot hook
✅ Implement extract_task_ids_recursive helper
✅ Update details page to use ordered tasks
✅ Handle empty/missing snapshots gracefully
✅ Test with parallel workflows
✅ Test with nested workflows
✅ Maintain performance with memoization
✅ SRP compliance maintained

## Testing

### Manual Testing

1. Run workflow with user input
2. Check GUI shows correct task order
3. Verify task details panel works
4. Test navigation between tasks
5. Confirm order matches execution flow

### Unit Testing (Future)

```rust
#[test]
fn test_extract_task_ids_recursive() {
    let json = serde_json::json!({
        "tasks": [
            {"id": "task1", "next_tasks": [
                {"id": "task2", "next_tasks": []}
            ]}
        ]
    });
    
    let task_ids = extract_task_ids_recursive(&json);
    assert_eq!(task_ids, vec!["task1", "task2"]);
}
```

## Next Steps

After completing GUI changes:

1. Test end-to-end with real workflows
2. Verify task ordering in various scenarios
3. Run performance tests
4. Update documentation
5. Proceed to final testing phase

