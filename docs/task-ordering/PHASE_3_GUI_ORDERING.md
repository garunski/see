# Phase 3: GUI Ordering - Implement Task Ordering from Snapshot

## Objective

Implement task ordering logic in GUI using workflow snapshot to display tasks in correct execution order.

## Status: ⏳ Pending

## Files to Modify

### 1. gui/src/pages/executions/details/hooks.rs

Add new hook and helper function:

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

### 2. gui/src/pages/executions/details/page.rs

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

                // ... rest of page unchanged ...
            }
        }

        // Task Details Panel - unchanged, works with reordered tasks
        TaskDetailsPanel {
            is_open: is_panel_open(),
            current_task: execution().and_then(|exec| exec.tasks.get(selected_task_index()).map(s_e_e_core::task_execution_to_info)),
            current_task_index: selected_task_index(),
            total_tasks: execution().map(|exec| exec.tasks.len()).unwrap_or(0),
            execution: execution(),
            on_close: move |_| is_panel_open.set(false),
            on_previous: move |_| {
                let current = selected_task_index();
                if current > 0 {
                    selected_task_index.set(current - 1);
                }
            },
            on_next: move |_| {
                let current = selected_task_index();
                let total = execution().map(|exec| exec.tasks.len()).unwrap_or(0);
                if current < total.saturating_sub(1) {
                    selected_task_index.set(current + 1);
                }
            }
        }
    }
}
```

## Implementation Steps

1. ⏳ Open `gui/src/pages/executions/details/hooks.rs`
2. ⏳ Add `use_task_order_from_snapshot` hook
3. ⏳ Add `extract_task_ids_recursive` helper
4. ⏳ Update `gui/src/pages/executions/details/page.rs`
5. ⏳ Add task reordering logic
6. ⏳ Build GUI: `cargo build -p s_e_e_gui`

## Testing

### Manual Testing

1. **Start GUI**:
   ```bash
   cargo run -p s_e_e_gui
   ```

2. **Execute a workflow**:
   - Create or load a workflow
   - Execute it
   - View execution details

3. **Verify task order**:
   - Check tasks display in correct execution order
   - Navigate between tasks
   - Verify details panel shows correct task

### Test Workflows

Test with various workflows:

**Simple sequential**:
```json
{"tasks": [{"id": "task1", "next_tasks": [{"id": "task2", "next_tasks": []}]}]}
```
Expected order: `["task1", "task2"]`

**Parallel tasks**:
```json
{"tasks": [{"id": "task1", "next_tasks": []}, {"id": "task2", "next_tasks": []}]}
```
Expected order: `["task1", "task2"]`

**Nested tasks**:
```json
{"tasks": [{"id": "task1", "next_tasks": [{"id": "task2", "next_tasks": [{"id": "task3", "next_tasks": []}]}]}]}
```
Expected order: `["task1", "task2", "task3"]`

## Validation Checklist

- [ ] Code compiles without errors
- [ ] Tasks display in correct order
- [ ] Navigation works (previous/next)
- [ ] Panel works with reordered tasks
- [ ] No performance issues
- [ ] Edge cases handled (empty snapshot, missing tasks)

## Edge Case Handling

### Empty Snapshot

If snapshot is `{}` (shouldn't happen):
```rust
// Hook returns empty vec
let ordered_task_ids = use_task_order_from_snapshot(execution);
if ordered_task_ids().is_empty() {
    // Fall back to exec.tasks
    exec.tasks.iter().map(...).collect()
}
```

### Missing Tasks

Tasks in snapshot not present in execution:
```rust
.filter_map(|id| task_map.get(id).cloned())
```
Simply omitted from display.

## SRP Compliance

- ✅ hooks.rs only contains hook definitions
- ✅ page.rs only contains page component
- ✅ components/ separated by responsibility
- ✅ Each file has one responsibility

## Next Phase

After completion, proceed to **Phase 4: Bug Investigation**.

