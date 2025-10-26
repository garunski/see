# Task Ordering - Persistence Layer Specification

## Overview

This specification describes the persistence layer changes required to support workflow snapshot storage for task ordering.

## Model Changes

### WorkflowExecution Model

**File**: `persistence/src/models/execution.rs`

Add new field to store workflow JSON snapshot:

```rust
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub workflow_snapshot: serde_json::Value,  // NEW
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

### Field Details

**`workflow_snapshot: serde_json::Value`**
- Type: `serde_json::Value` (not String)
- Purpose: Store complete workflow JSON structure at execution time
- Contents: Original workflow JSON from `WorkflowDefinition.content`
- Serialization: Handled by serde_json automatically
- Storage: Stored as JSON in `workflow_executions.data` column

### Default Implementation

Update `Default` implementation to include empty JSON:

```rust
impl Default for WorkflowExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_name: String::new(),
            workflow_snapshot: serde_json::json!({}),  // Empty JSON object
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

## Database Schema

### Current Schema

No schema changes required. SQLite stores JSON as TEXT, and serde_json::Value serializes automatically.

**Table**: `workflow_executions`

```sql
CREATE TABLE IF NOT EXISTS workflow_executions (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL  -- Stores entire WorkflowExecution JSON
);
```

The new field is added to the serialized JSON automatically via serde.

### Storage Format

When serialized, the `workflow_snapshot` field appears in the JSON data:

```json
{
  "id": "exec-123",
  "workflow_name": "My Workflow",
  "workflow_snapshot": {
    "id": "workflow-123",
    "name": "My Workflow",
    "tasks": [...]
  },
  "status": "running",
  "tasks": [...],
  ...
}
```

## Serialization

### Automatic Serialization

Since we're using `serde_json::Value`:

- Serialization: `serde_json::to_string()` includes the field
- Deserialization: `serde_json::from_str()` includes the field
- No special handling needed

### Example

```rust
let execution = WorkflowExecution {
    id: "exec-1".to_string(),
    workflow_name: "Test".to_string(),
    workflow_snapshot: serde_json::from_str(workflow_json).unwrap(),
    // ... other fields
};

// Serialize (automatic)
let json = serde_json::to_string(&execution)?;

// Deserialize (automatic)
let execution: WorkflowExecution = serde_json::from_str(&json)?;
```

## Migration Strategy

### Clean Slate Approach

**No migration needed - database reset required**

Reason: Adding a required field would break existing executions. Clean slate approach:
1. Delete `data/data.db`
2. Recreate database on next run
3. All new executions include workflow_snapshot from start

### Alternative: Optional Field (If Needed Later)

If backward compatibility becomes necessary:

```rust
pub workflow_snapshot: Option<serde_json::Value>,
```

Then handle `None` in downstream code.

**Decision**: Use required field with clean slate for simplicity.

## Store Operations

### No Changes Needed

The existing store operations automatically handle the new field:

- `save_workflow_execution()` - Serializes whole struct including snapshot
- `get_workflow_execution()` - Deserializes whole struct including snapshot
- `list_workflow_executions()` - Returns complete structs

**No code changes required in store layer.**

## Test Updates

### Test Helper Functions

Update test helpers to include workflow_snapshot:

**File**: `persistence/tests/execution_tests.rs`

```rust
fn create_test_execution() -> WorkflowExecution {
    WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test Workflow",
            "tasks": []
        }),
        status: WorkflowStatus::Complete,
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
        success: Some(true),
        tasks: vec![],
        timestamp: Utc::now(),
        audit_trail: Vec::new(),
        per_task_logs: HashMap::new(),
        errors: Vec::new(),
    }
}
```

### Files to Update

1. `persistence/tests/execution_tests.rs` - `create_test_execution()`
2. `persistence/tests/store/execution_tests.rs` - `create_test_execution()`
3. `persistence/tests/models/execution_tests.rs` - Test structs
4. `persistence/tests/integration_tests.rs` - Execution creation
5. `persistence/tests/concurrency_tests.rs` - Concurrent creation

### Model Tests

Add test for serialization/deserialization with snapshot:

```rust
#[test]
fn test_workflow_execution_serialization_with_snapshot() {
    let execution = WorkflowExecution {
        id: "exec-1".to_string(),
        workflow_name: "Test".to_string(),
        workflow_snapshot: serde_json::json!({
            "id": "test",
            "name": "Test",
            "tasks": []
        }),
        // ... other fields
    };
    
    let json = serde_json::to_string(&execution).unwrap();
    let deserialized: WorkflowExecution = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.workflow_snapshot, execution.workflow_snapshot);
}
```

## SRP Compliance

### File Organization

Following Single Responsibility Principle:

- **models/execution.rs** - ONLY model definitions
- **store/execution.rs** - ONLY CRUD operations
- **tests/models/** - ONLY model tests
- **tests/store/** - ONLY store tests

Each file has ONE responsibility.

## Validation

### Requirements Checklist

✅ Add workflow_snapshot field to WorkflowExecution
✅ Update Default implementation
✅ Add to serialization tests
✅ Update all test helpers
✅ Database reset documented
✅ No store layer changes needed
✅ SRP compliance maintained

## Next Steps

After completing persistence changes:

1. Update core layer to store snapshot on execution
2. Update bridge layer for proper conversion
3. Test serialization/deserialization
4. Run persistence tests
5. Proceed to GUI layer implementation

