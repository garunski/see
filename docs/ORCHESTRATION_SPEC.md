# Orchestration Flow Specification

## Overview
This document defines the step-by-step execution flow for workflow orchestration in the core crate. It describes how workflows are loaded from persistence, executed through the engine, and results are saved back to persistence.

## Primary Execution Flows

### Flow 1: Workflow Execution (`execute_workflow_by_id`)

This is the main workflow execution flow that coordinates between persistence and engine.

#### Step 1: Load WorkflowDefinition from Persistence
```rust
let store = get_global_store()?;
let workflow = store.get_workflow(workflow_id).await
    .map_err(|e| CoreError::Persistence(e))?
    .ok_or_else(|| CoreError::WorkflowNotFound(workflow_id.to_string()))?;
```

**State:**
- Input: `workflow_id: &str`
- Output: `WorkflowDefinition`
- Errors: `CoreError::Persistence` or `CoreError::WorkflowNotFound`

**Validation:**
- Check workflow exists in database
- Deserialize successfully from storage

---

#### Step 2: Validate Workflow Content
```rust
if workflow.content.is_empty() {
    return Err(CoreError::Execution("Workflow content is empty".to_string()));
}

// Validate JSON is parseable
serde_json::from_str::<serde_json::Value>(&workflow.content)
    .map_err(|e| CoreError::Execution(format!("Invalid workflow JSON: {}", e)))?;
```

**State:**
- Input: `WorkflowDefinition`
- Output: Validated workflow
- Errors: `CoreError::Execution` if content is empty or invalid JSON

**Validation:**
- Content field is not empty
- Content is valid JSON
- JSON has required fields (id, name, tasks)

---

#### Step 3: Parse JSON Content to EngineWorkflow
```rust
let engine_workflow = engine::parse_workflow(&workflow.content)
    .map_err(|e| CoreError::Engine(e.into()))?;
```

**State:**
- Input: `workflow.content: String` (JSON)
- Output: `EngineWorkflow`
- Errors: `CoreError::Engine` with parser error

**Type Conversion:**
- JSON string → `EngineWorkflow` struct
- Validates task structure and dependencies
- See BRIDGE_SPEC.md for field mappings

---

#### Step 4: Create Initial WorkflowExecution Record
```rust
let execution_id = uuid::Uuid::new_v4().to_string();
let now = chrono::Utc::now();

let initial_execution = WorkflowExecution {
    id: execution_id.clone(),
    workflow_name: workflow.name.clone(),
    status: WorkflowStatus::Running,
    created_at: now,
    completed_at: None,
    success: false,
    tasks: Vec::new(),  // Will be populated after execution
    timestamp: now,
};
```

**State:**
- Input: `WorkflowDefinition`
- Output: `WorkflowExecution` with status `Running`
- New Data: Generated `execution_id` (UUID v4)

**Fields:**
- `id`: New UUID
- `workflow_name`: From workflow definition
- `status`: `WorkflowStatus::Running`
- `created_at`: Current timestamp
- `completed_at`: None (not finished yet)
- `success`: false (not finished yet)
- `tasks`: Empty vector (populated later)
- `timestamp`: Current timestamp

---

#### Step 5: Save Initial Execution to Persistence
```rust
store.save_workflow_execution(initial_execution.clone()).await
    .map_err(|e| CoreError::Persistence(e))?;
```

**State:**
- Input: `WorkflowExecution` with status `Running`
- Output: Persisted execution record
- Errors: `CoreError::Persistence` if save fails

**Purpose:**
- Record execution start time
- Create execution record GUI can query
- Provide execution_id for tracking

---

#### Step 6: Execute Workflow Through Engine
```rust
let engine = engine::WorkflowEngine::new();
let engine_result = engine.execute_workflow(engine_workflow).await
    .map_err(|e| CoreError::Engine(e))?;
```

**State:**
- Input: `EngineWorkflow`
- Output: `engine::WorkflowResult`
- Errors: `CoreError::Engine` if execution fails

**Engine Actions:**
- Resolves task dependencies
- Executes tasks in parallel where possible
- Calls output callback for progress updates
- Generates audit trail
- Collects per-task logs
- Returns execution results

---

#### Step 7: Stream Progress via OutputCallback
```rust
// Callback is passed to engine during execution
if let Some(ref callback) = callback {
    callback("Task 1 starting...".to_string());
    callback("Task 1 complete.".to_string());
}
```

**State:**
- Input: `Option<OutputCallback>`
- Output: Real-time progress messages
- Called By: Engine during task execution

**Messages:**
- Task start/completion events
- Task output streams
- Error messages
- Status updates

**Note:** Engine handles callback invocation internally during `execute_workflow`.

---

#### Step 8: Convert Engine Result to Persistence Types
```rust
// Convert WorkflowResult → WorkflowExecution + TaskExecution[]
let completed_at = chrono::Utc::now();

let task_executions = convert_task_info_to_executions(
    &engine_result.tasks,
    &execution_id,
    &engine_result.per_task_logs,
    &engine_result.errors,
);

let final_execution = WorkflowExecution {
    id: execution_id.clone(),
    workflow_name: engine_result.workflow_name.clone(),
    status: if engine_result.success {
        WorkflowStatus::Complete
    } else {
        WorkflowStatus::Failed
    },
    created_at: initial_execution.created_at,
    completed_at: Some(completed_at),
    success: engine_result.success,
    tasks: task_executions,
    timestamp: completed_at,
};
```

**State:**
- Input: `engine::WorkflowResult`
- Output: `WorkflowExecution` with status `Complete` or `Failed`
- Type Conversion: See BRIDGE_SPEC.md for detailed mappings

**Field Updates:**
- `status`: `Running` → `Complete` or `Failed`
- `completed_at`: Set to current timestamp
- `success`: From engine result
- `tasks`: Converted from engine TaskInfo[]
- `timestamp`: Updated to completion time

---

#### Step 9: Save Task Executions to Persistence
```rust
for task in &final_execution.tasks {
    store.save_task_execution(task.clone()).await
        .map_err(|e| CoreError::Persistence(e))?;
}
```

**State:**
- Input: `Vec<TaskExecution>`
- Output: Persisted task records
- Errors: `CoreError::Persistence` if save fails

**Purpose:**
- Store individual task results
- Enable task-level queries
- Support execution history

---

#### Step 10: Save Audit Events to Persistence
```rust
for audit_entry in &engine_result.audit_trail {
    let audit_event = convert_audit_entry_to_event(audit_entry);
    store.log_audit_event(audit_event).await
        .map_err(|e| CoreError::Persistence(e))?;
}
```

**State:**
- Input: `Vec<AuditEntry>` from engine
- Output: Persisted `AuditEvent` records
- Type Conversion: See BRIDGE_SPEC.md

**Purpose:**
- Store audit trail
- Track workflow changes
- Support compliance/debugging

---

#### Step 11: Update Final Execution Record
```rust
store.save_workflow_execution(final_execution.clone()).await
    .map_err(|e| CoreError::Persistence(e))?;
```

**State:**
- Input: `WorkflowExecution` with final status
- Output: Updated execution record in database
- Errors: `CoreError::Persistence` if save fails

**Updates:**
- Status changed to Complete/Failed
- Completion timestamp set
- Success flag set
- Tasks populated

---

#### Step 12: Return WorkflowResult
```rust
// Create enhanced WorkflowResult with execution_id
let result = WorkflowResult {
    success: engine_result.success,
    workflow_name: engine_result.workflow_name,
    execution_id: execution_id,  // Added for GUI tracking
    tasks: engine_result.tasks,
    audit_trail: engine_result.audit_trail,
    per_task_logs: engine_result.per_task_logs,
    errors: engine_result.errors,
};

Ok(result)
```

**State:**
- Input: Engine result + execution metadata
- Output: `WorkflowResult` with `execution_id`
- Return Type: `Result<WorkflowResult, CoreError>`

**Result Contents:**
- `success`: Overall execution success
- `workflow_name`: Name of executed workflow
- `execution_id`: UUID for database lookup
- `tasks`: Task information for GUI display
- `audit_trail`: Audit events
- `per_task_logs`: Detailed task output
- `errors`: Collected error messages

---

### Complete Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ execute_workflow_by_id(workflow_id, callback)               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 1: Load WorkflowDefinition from Persistence            │
│   store.get_workflow(workflow_id)                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 2: Validate Workflow Content                           │
│   Check JSON is valid and not empty                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 3: Parse JSON → EngineWorkflow                         │
│   engine::parse_workflow(&content)                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 4: Create Initial WorkflowExecution                    │
│   status: Running, tasks: [], success: false                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 5: Save Initial Execution to DB                        │
│   store.save_workflow_execution(initial)                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 6: Execute Through Engine                              │
│   engine.execute_workflow(engine_workflow)                  │
│   → Callback invoked during execution                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 8: Convert Engine Results                              │
│   WorkflowResult → WorkflowExecution + TaskExecution[]      │
│   status: Complete/Failed, tasks: populated                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 9: Save Task Executions                                │
│   for task: store.save_task_execution(task)                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 10: Save Audit Events                                  │
│   for audit: store.log_audit_event(audit)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 11: Update Final Execution Record                      │
│   store.save_workflow_execution(final)                      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 12: Return WorkflowResult                              │
│   { success, execution_id, tasks, audit_trail, ... }        │
└─────────────────────────────────────────────────────────────┘
```

---

## Flow 2: Task Resumption (`resume_task`)

This flow handles resuming a paused task that's waiting for input.

#### Step 1: Load WorkflowExecution from Persistence
```rust
let store = get_global_store()?;
let execution = store.get_workflow_execution(execution_id).await
    .map_err(|e| CoreError::Persistence(e))?
    .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;
```

**State:**
- Input: `execution_id: &str`
- Output: `WorkflowExecution`
- Errors: `CoreError::Persistence` or `CoreError::WorkflowNotFound`

---

#### Step 2: Find TaskExecution by task_id
```rust
let task = execution.tasks.iter()
    .find(|t| t.id == task_id)
    .ok_or_else(|| CoreError::TaskNotFound(task_id.to_string()))?;
```

**State:**
- Input: `task_id: &str`, `execution.tasks: Vec<TaskExecution>`
- Output: `&TaskExecution`
- Errors: `CoreError::TaskNotFound` if task doesn't exist

---

#### Step 3: Validate Task Status
```rust
if task.status != TaskStatus::WaitingForInput {
    return Err(CoreError::Execution(
        format!("Task {} is not waiting for input (status: {:?})", task_id, task.status)
    ));
}
```

**State:**
- Input: `TaskExecution`
- Output: Validated task
- Errors: `CoreError::Execution` if status is not `WaitingForInput`

**Valid Status Transition:**
- From: `WaitingForInput`
- To: `InProgress` → `Complete` or `Failed`

---

#### Step 4: Resume Task Execution via Engine
```rust
// Note: Engine doesn't currently support task resumption
// This is a placeholder for future implementation

// Future implementation:
// let engine = engine::WorkflowEngine::new();
// let result = engine.resume_task(execution_id, task_id).await?;
```

**State:**
- Input: `execution_id`, `task_id`
- Output: Task execution result
- Errors: `CoreError::Engine` if resumption fails

**Note:** This feature is not yet implemented in the engine. The orchestration layer is prepared to support it once engine adds the capability.

---

#### Step 5: Update Task Status in Persistence
```rust
let mut updated_task = task.clone();
updated_task.status = TaskStatus::Complete;
updated_task.completed_at = Some(chrono::Utc::now());

store.save_task_execution(updated_task).await
    .map_err(|e| CoreError::Persistence(e))?;
```

**State:**
- Input: Updated `TaskExecution`
- Output: Persisted task with new status
- Errors: `CoreError::Persistence` if save fails

---

#### Step 6: Update Execution if All Tasks Complete
```rust
let all_tasks_complete = execution.tasks.iter()
    .all(|t| matches!(t.status, TaskStatus::Complete | TaskStatus::Failed));

if all_tasks_complete {
    let mut updated_execution = execution.clone();
    updated_execution.status = WorkflowStatus::Complete;
    updated_execution.completed_at = Some(chrono::Utc::now());
    
    store.save_workflow_execution(updated_execution).await
        .map_err(|e| CoreError::Persistence(e))?;
}
```

**State:**
- Input: Updated execution
- Output: Final execution record if workflow complete
- Conditional: Only if all tasks finished

---

#### Step 7: Return Success
```rust
Ok(())
```

**State:**
- Output: `Result<(), CoreError>`

---

## State Transitions

### WorkflowStatus State Machine

```
┌─────────┐
│ Pending │ (Initial state, not used in current implementation)
└────┬────┘
     │
     ▼
┌─────────┐
│ Running │ (Set when execute_workflow_by_id starts)
└────┬────┘
     │
     ├──────────┐
     ▼          ▼
┌──────────┐ ┌────────┐
│ Complete │ │ Failed │ (Final states)
└──────────┘ └────────┘
```

**Transitions:**
1. `Pending` → `Running`: Workflow execution starts
2. `Running` → `Complete`: All tasks succeed
3. `Running` → `Failed`: Any task fails or engine error

**Current Implementation:**
- Workflows start directly in `Running` state
- No automatic `Pending` state
- GUI could create `Pending` executions manually for queuing

---

### TaskStatus State Machine

```
┌─────────┐
│ Pending │ (Task queued, waiting for dependencies)
└────┬────┘
     │
     ▼
┌────────────┐
│ InProgress │ (Task executing)
└─────┬──────┘
      │
      ├──────────┬────────────────┬──────────────────┐
      ▼          ▼                ▼                  ▼
┌──────────┐ ┌────────┐ ┌─────────────────┐ ┌────────┐
│ Complete │ │ Failed │ │ WaitingForInput │ │ Failed │
└──────────┘ └────────┘ └────────┬────────┘ └────────┘
                                 │
                                 │ resume_task()
                                 ▼
                        ┌────────────┐
                        │ InProgress │
                        └─────┬──────┘
                              │
                              ├─────────┬────────┐
                              ▼         ▼        ▼
                        ┌──────────┐ ┌────────┐
                        │ Complete │ │ Failed │
                        └──────────┘ └────────┘
```

**Transitions:**
1. `Pending` → `InProgress`: Task starts executing
2. `InProgress` → `Complete`: Task succeeds
3. `InProgress` → `Failed`: Task fails
4. `InProgress` → `WaitingForInput`: Task needs user input
5. `WaitingForInput` → `InProgress`: Task resumed via `resume_task()`

---

## Persistence Points

### 1. Before Execution Starts
**When:** After Step 5 in workflow execution
**What:** Initial `WorkflowExecution` record
**Purpose:**
- Record execution attempt
- Provide execution_id for tracking
- Allow GUI to show "Running" status immediately

**Data Saved:**
```rust
WorkflowExecution {
    status: Running,
    created_at: now,
    completed_at: None,
    success: false,
    tasks: [],
}
```

---

### 2. During Execution (Optional/Future)
**When:** After each task completes
**What:** Incremental task results
**Purpose:**
- Show real-time progress in GUI
- Support task-level resumption
- Preserve partial results if crash occurs

**Data Saved:**
```rust
TaskExecution {
    status: Complete/Failed,
    completed_at: Some(now),
    output: Some(output_text),
    error: Some(error_text),
}
```

**Note:** Not currently implemented. Engine completes entire workflow before returning. Future enhancement would stream task results.

---

### 3. After Execution Completes
**When:** After Step 11 in workflow execution
**What:** Final execution results
**Purpose:**
- Record completion status
- Store all task results
- Store audit trail
- Enable execution history queries

**Data Saved:**
```rust
WorkflowExecution {
    status: Complete/Failed,
    completed_at: Some(now),
    success: true/false,
    tasks: [...],  // All task results
}

Vec<TaskExecution> { ... }  // Individual task records
Vec<AuditEvent> { ... }  // Audit trail
```

---

## Type Conversions at Each Step

### Load: Database → Persistence Types

**Conversion:** `redb` Value → Rust struct

```rust
// redb stores serialized JSON
let value: &[u8] = table.get(key)?;
let workflow: WorkflowDefinition = serde_json::from_slice(value)?;
```

**Format:** JSON serialization via serde
**Traits:** `Serialize`, `Deserialize`

---

### Parse: WorkflowDefinition.content → EngineWorkflow

**Conversion:** JSON string → `EngineWorkflow`

```rust
let engine_workflow = engine::parse_workflow(&workflow.content)?;
```

**Mapping:**
```
WorkflowDefinition.content (String) → EngineWorkflow
{
  "id": "...",
  "name": "...",
  "tasks": [...]
}
→
EngineWorkflow {
  id: String,
  name: String,
  tasks: Vec<EngineTask>,
}
```

**Details:** See BRIDGE_SPEC.md section "WorkflowDefinition → EngineWorkflow"

---

### Execute: EngineWorkflow → WorkflowResult

**Conversion:** Engine execution → Result struct

```rust
let result: WorkflowResult = engine.execute_workflow(engine_workflow).await?;
```

**Output:**
```rust
WorkflowResult {
    success: bool,
    workflow_name: String,
    tasks: Vec<TaskInfo>,
    audit_trail: Vec<AuditEntry>,
    per_task_logs: HashMap<String, Vec<String>>,
    errors: Vec<String>,
}
```

**No conversion needed** - Engine already returns this type.

---

### Save: WorkflowResult → WorkflowExecution + TaskExecution[]

**Conversion:** Engine result → Persistence types

**Main Conversion:**
```rust
WorkflowResult → WorkflowExecution {
    id: generated_uuid,
    workflow_name: result.workflow_name,
    status: if result.success { Complete } else { Failed },
    created_at: saved_timestamp,
    completed_at: Some(now),
    success: result.success,
    tasks: convert_tasks(result.tasks),
    timestamp: now,
}
```

**Task Conversion:**
```rust
Vec<TaskInfo> → Vec<TaskExecution>

For each TaskInfo:
  TaskExecution {
      id: task.id,
      workflow_id: execution_id,
      name: task.name,
      status: task.status,
      output: extract_from_per_task_logs,
      error: extract_from_errors,
      created_at: estimate_or_unknown,
      completed_at: estimate_or_unknown,
  }
```

**Details:** See BRIDGE_SPEC.md for complete mappings.

---

## Error Handling at Each Step

### Persistence Errors

**Sources:**
- Database connection failures
- Serialization/deserialization errors
- Disk I/O errors
- Transaction errors

**Handling:**
```rust
store.save_workflow_execution(exec).await
    .map_err(|e| CoreError::Persistence(e))?;
```

**Conversion:** `String` → `CoreError::Persistence(String)`

**Recovery:** No automatic recovery. Error propagates to caller.

---

### Engine Errors

**Sources:**
- Workflow parsing errors (invalid JSON, missing fields)
- Task execution errors (handler failures)
- Graph errors (circular dependencies)

**Handling:**
```rust
engine.execute_workflow(wf).await
    .map_err(|e| CoreError::Engine(e))?;
```

**Conversion:** `EngineError` → `CoreError::Engine(EngineError)`

**Recovery:** No automatic recovery. Error propagates to caller. Execution record marked as failed.

---

### Not Found Errors

**Sources:**
- Workflow ID doesn't exist in database
- Task ID doesn't exist in execution

**Handling:**
```rust
store.get_workflow(id).await?
    .ok_or_else(|| CoreError::WorkflowNotFound(id.to_string()))?
```

**Recovery:** No automatic recovery. User must provide valid ID.

---

### Validation Errors

**Sources:**
- Empty workflow content
- Invalid JSON format
- Invalid task status for operation

**Handling:**
```rust
if workflow.content.is_empty() {
    return Err(CoreError::Execution("Workflow content is empty".to_string()));
}
```

**Recovery:** No automatic recovery. Validation must pass.

---

### Partial Failure Handling

**Scenario:** Workflow executes but save fails

**Strategy:**
1. Engine execution completes successfully
2. Persistence save fails (disk full, etc.)
3. Error returned to caller
4. Initial execution record shows "Running" status
5. No final results saved

**Consequences:**
- GUI shows workflow as running forever
- Results lost
- Manual cleanup needed

**Future Enhancement:**
- Retry logic for persistence saves
- Write results to temporary file as backup
- Recovery mechanism to find orphaned executions

---

### Error Propagation Chain

```
Engine Error
    ↓
CoreError::Engine(EngineError)
    ↓
GUI Service Error
    ↓
GUI displays error message
```

```
Persistence Error (String)
    ↓
CoreError::Persistence(String)
    ↓
GUI Service Error
    ↓
GUI displays error message
```

---

## Concurrency Considerations

### Parallel Execution Support

**Engine Side:**
- Engine executes independent tasks in parallel
- Uses Tokio async runtime
- Tasks with dependencies execute sequentially

**Orchestration Side:**
- `execute_workflow_by_id` is async
- Can be called concurrently for different workflows
- Each execution is independent

**Persistence Side:**
- Store uses async Tokio RwLock
- Multiple writes are serialized by lock
- Multiple reads are concurrent

---

### Multi-Process Safety

**Scenario:** Multiple GUI processes running

**Database:**
- `redb` supports multi-process readers
- Single writer per database (handled by redb)
- Readers don't block each other
- Writers don't block readers (MVCC)

**Store Singleton:**
- Each process has its own `Arc<Store>`
- All share same database file
- Safe concurrent access

**Execution Isolation:**
- Each execution has unique UUID
- No conflict between concurrent executions
- Results saved independently

---

## Performance Characteristics

### Execution Latency Breakdown

1. **Load WorkflowDefinition:** ~1-5ms (database read)
2. **Parse JSON:** ~1-10ms (depends on workflow size)
3. **Create Initial Execution:** <1ms (struct creation)
4. **Save Initial Execution:** ~5-20ms (database write)
5. **Engine Execution:** Variable (depends on tasks)
6. **Convert Results:** ~1-5ms (struct conversions)
7. **Save Task Executions:** ~5-20ms per task (database writes)
8. **Save Audit Events:** ~5-20ms per event (database writes)
9. **Update Final Execution:** ~5-20ms (database write)

**Total Overhead:** ~50-200ms (excluding engine execution)

---

### Memory Usage

**WorkflowExecution:**
- Base struct: ~200 bytes
- Tasks vector: ~100 bytes per task
- Strings: Variable (names, IDs, errors)

**Typical Workflow:**
- 10 tasks: ~2KB
- 100 tasks: ~15KB

**Engine Execution:**
- Task contexts: Variable
- Output logs: Variable (can be large)
- Audit trail: ~500 bytes per entry

---

## Testing Strategy

### Unit Tests

**Test Each Step Individually:**
```rust
#[tokio::test]
async fn test_load_workflow() {
    let store = setup_test_store().await;
    let workflow = store.get_workflow("test-id").await.unwrap();
    assert_eq!(workflow.id, "test-id");
}

#[tokio::test]
async fn test_parse_workflow() {
    let json = r#"{"id":"1","name":"test","tasks":[]}"#;
    let result = engine::parse_workflow(json);
    assert!(result.is_ok());
}
```

---

### Integration Tests

**Test Complete Flow:**
```rust
#[tokio::test]
async fn test_execute_workflow_end_to_end() {
    init_global_store().await.unwrap();
    
    // Create and save workflow
    let workflow = create_test_workflow();
    get_global_store().unwrap().save_workflow(&workflow).await.unwrap();
    
    // Execute
    let result = execute_workflow_by_id(&workflow.id, None).await.unwrap();
    
    // Verify
    assert!(result.success);
    assert!(!result.execution_id.is_empty());
    
    // Check persistence
    let execution = get_global_store().unwrap()
        .get_workflow_execution(&result.execution_id).await.unwrap();
    assert_eq!(execution.status, WorkflowStatus::Complete);
}
```

---

### Error Tests

**Test Error Paths:**
```rust
#[tokio::test]
async fn test_workflow_not_found() {
    init_global_store().await.unwrap();
    
    let result = execute_workflow_by_id("nonexistent", None).await;
    
    assert!(matches!(result, Err(CoreError::WorkflowNotFound(_))));
}

#[tokio::test]
async fn test_invalid_workflow_json() {
    let workflow = WorkflowDefinition {
        content: "invalid json".to_string(),
        ..Default::default()
    };
    
    get_global_store().unwrap().save_workflow(&workflow).await.unwrap();
    
    let result = execute_workflow_by_id(&workflow.id, None).await;
    assert!(result.is_err());
}
```

---

## Success Criteria

✓ Workflow execution flow is complete and detailed
✓ Task resumption flow is documented
✓ All state transitions are defined
✓ Persistence points are identified
✓ Type conversions are specified at each step
✓ Error handling is comprehensive
✓ Concurrency model is clear
✓ Performance characteristics are documented
✓ Testing strategy is defined

