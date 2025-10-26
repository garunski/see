# Fix Task Order by Storing Workflow Snapshot in Execution

## Problem

Tasks are displayed in the wrong order because:
1. Tasks are flattened from the workflow JSON during parsing (DFS order)
2. The order they appear in `exec.tasks` doesn't match execution order
3. We can't determine the correct order without the original workflow structure

## Solution

Store a snapshot of the workflow JSON in the `WorkflowExecution` to preserve the exact structure that was executed.

## Changes Required

### 1. Update WorkflowExecution Model

**File:** `persistence/src/models/execution.rs`

Add a field to store the workflow snapshot:

```rust
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub workflow_snapshot: String,  // NEW: Store workflow JSON snapshot
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub tasks: Vec<TaskExecution>,
    pub timestamp: DateTime<Utc>,
    pub audit_trail: Vec<AuditEvent>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}
```

### 2. Save Workflow Snapshot When Creating Execution

**File:** `core/src/api/execution.rs`

When creating the initial execution (line 45), include the workflow snapshot:

```rust
let initial_execution = WorkflowExecution {
    id: execution_id.clone(),
    workflow_name: workflow.name.clone(),
    workflow_snapshot: workflow.content.clone(),  // NEW: Store snapshot
    status: WorkflowStatus::Running,
    created_at: now,
    completed_at: None,
    success: None,
    tasks: Vec::new(),
    timestamp: now,
    audit_trail: Vec::new(),
    per_task_logs: std::collections::HashMap::new(),
    errors: Vec::new(),
};
```

Also preserve it when updating:

```rust
// Line 82: Update execution status to indicate waiting
let mut updated_execution = initial_execution.clone();
updated_execution.workflow_snapshot = workflow.content.clone();  // Preserve snapshot
updated_execution.status = WorkflowStatus::Running;
```

### 3. Create Helper to Extract Task Order from Snapshot

**File:** `gui/src/pages/executions/details/hooks.rs`

Add a new hook to parse task order from workflow snapshot:

```rust
use serde_json::Value;

/// Extract task IDs in order from workflow snapshot
pub fn use_task_order_from_snapshot(
    execution: Signal<Option<WorkflowExecution>>,
) -> Memo<Vec<String>> {
    use_signal(|| Vec::new());
    
    let ordered_task_ids = use_memo(move || {
        if let Some(exec) = execution() {
            if let Ok(json) = serde_json::from_str::<Value>(&exec.workflow_snapshot) {
                extract_task_ids_recursive(&json)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    });
    
    ordered_task_ids
}

/// Recursively extract task IDs from workflow JSON preserving order
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

### 4. Use Ordered Tasks in Details Page

**File:** `gui/src/pages/executions/details/page.rs`

Update to use ordered tasks:

```rust
let (execution, loading, error) = use_workflow_execution(id.clone());
let ordered_task_ids = use_task_order_from_snapshot(execution);

// Reorder tasks based on workflow snapshot
let ordered_tasks = if let Some(exec) = execution() {
    let mut task_map: HashMap<String, _> = exec.tasks.iter()
        .map(|t| (t.id.clone(), t.clone()))
        .collect();
    
    ordered_task_ids().iter()
        .filter_map(|task_id| task_map.get(task_id))
        .cloned()
        .collect()
} else {
    Vec::new()
};

// Use ordered_tasks instead of exec.tasks for rendering
```

### 5. Extract Workflow Details from Snapshot for UI Display

**File:** `gui/src/pages/executions/details/hooks.rs` (or create new utility)

Add helper to extract workflow metadata from snapshot:

```rust
use serde_json::Value;

/// Get workflow name, ID, and other details from snapshot
pub fn use_workflow_details_from_snapshot(
    execution: Signal<Option<WorkflowExecution>>,
) -> Memo<Option<serde_json::Value>> {
    use_signal(|| None);
    
    let workflow_details = use_memo(move || {
        if let Some(exec) = execution() {
            serde_json::from_str::<Value>(&exec.workflow_snapshot).ok()
        } else {
            None
        }
    });
    
    workflow_details
}

/// Get workflow name from snapshot
pub fn get_workflow_name_from_snapshot(snapshot: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(snapshot) {
        value.get("name")?.as_str().map(|s| s.to_string())
    } else {
        None
    }
}
```

**File:** `gui/src/pages/executions/details/page.rs`

Update to use workflow details from snapshot:

```rust
let workflow_details = use_workflow_details_from_snapshot(execution);

// Display workflow name from snapshot if available
let workflow_name = if let Some(details) = workflow_details() {
    details.get("name")?.as_str()?.to_string()
} else {
    exec.workflow_name.clone()
};
```

### 6. Update Default Implementation

**File:** `persistence/src/models/execution.rs`

Add the new field to default:

```rust
impl Default for WorkflowExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_name: String::new(),
            workflow_snapshot: String::new(),  // NEW: Empty snapshot
            status: WorkflowStatus::Pending,
            created_at: now,
            completed_at: None,
            success: None,
            tasks: Vec::new(),
            timestamp: now,
            audit_trail: Vec::new(),
            per_task_logs: HashMap::new(),
            errors: Vec::new(),
        }
    }
}
```

## Benefits

1. **Audit Trail**: Exact workflow structure is preserved as snapshot
2. **Correct Ordering**: Can determine task execution order from workflow structure  
3. **Reproducibility**: Can replay exact workflow that was executed
4. **Workflow Details**: Can extract workflow metadata (name, ID, structure) from snapshot
5. **Self-Contained**: Execution contains everything needed to display correctly
6. **No Reverse Lookup**: Don't need to query workflow definition separately

## Database Reset Required

**Clear existing database - no migration**

Since we're adding a new required field:
- Delete `data/data.db` file to reset database
- All new executions will include workflow_snapshot from the start
- No backward compatibility needed - clean slate approach

## Files to Modify

1. `persistence/src/models/execution.rs` - Add workflow_snapshot field
2. `core/src/api/execution.rs` - Store snapshot when creating execution
3. `gui/src/pages/executions/details/hooks.rs` - Add hooks to:
   - Extract task order from snapshot
   - Extract workflow details from snapshot
4. `gui/src/pages/executions/details/page.rs` - Use ordered tasks and workflow details from snapshot

## Workflow Details Extraction

The `workflow_snapshot` field stores the complete workflow JSON, allowing us to:
1. Extract workflow metadata (name, ID, description)
2. Get task execution order
3. Display workflow structure
4. Show exact workflow definition that was executed

This eliminates the need to:
- Query workflow definitions separately
- Match execution to workflow by name/ID
- Handle cases where workflow might have been deleted or modified

## Testing

1. **Clear Database**: Delete `data/data.db` to start fresh
2. Create new execution with workflow snapshot  
3. Verify tasks display in correct order from workflow JSON
4. Extract and display workflow name from snapshot in UI
5. Test with parallel and nested workflows  
6. Verify workflow details page shows correct ordering
7. Confirm execution is self-contained (no external lookups needed)
