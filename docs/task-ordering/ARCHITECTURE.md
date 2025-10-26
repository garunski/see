# Task Ordering and Workflow Snapshot - Architecture

## Overview

This document describes the architecture for adding workflow snapshot capability to preserve task execution order and fix the critical bug where workflows don't pause for user input.

## Problem Statement

### 1. Task Ordering Issue

**Symptom**: Tasks are displayed in the wrong order in the GUI

**Root Cause**: 
- Workflows are parsed using DFS (depth-first search) which flattens the hierarchical structure
- The order in `exec.tasks` doesn't match the actual execution order
- Without the original workflow JSON structure, correct ordering cannot be determined

**Example**:
```json
{
  "tasks": [
    {"id": "task1", "next_tasks": [
      {"id": "task2", "next_tasks": [
        {"id": "task3", "next_tasks": []}
      ]}
    ]}
  ]
}
```

When flattened, becomes: `["task1", "task2", "task3"]` but execution might be `["task1", "task2"]` in a different order.

### 2. Critical Bug: No Workflow Pause

**Symptom**: Workflows complete instead of pausing for user input

**Expected Behavior**:
1. User input task executes
2. Task status set to `WaitingForInput`
3. Engine pauses execution
4. Core API detects pause and returns early
5. GUI shows "waiting for input" status

**Actual Behavior**:
1. User input task executes
2. Workflow continues execution
3. All tasks complete
4. No pause mechanism activates

## System Architecture

### Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         GUI Layer                             │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │  Details Page    │  │  History Page    │                 │
│  │  - Task List     │  │  - Execution List│                 │
│  │  - Ordering ❌   │  │                  │                 │
│  └──────────────────┘  └──────────────────┘                 │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                         Core Layer                           │
│  ┌──────────────────────┐  ┌──────────────────────┐        │
│  │  execution.rs       │  │  resume.rs           │        │
│  │  - execute_workflow  │  │  - resume_task        │        │
│  │  - pause detection  │  │                      │        │
│  │  ❌ Not working     │  │                      │        │
│  └──────────────────────┘  └──────────────────────┘        │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                        Engine Layer                          │
│  ┌──────────────────────┐  ┌──────────────────────┐        │
│  │  engine.rs          │  │  user_input.rs       │        │
│  │  - execution loop    │  │  - handler           │        │
│  │  - result process   │  │  - return waiting   │        │
│  │  ❌ Bug here        │  │  ✅ Returns correct │        │
│  └──────────────────────┘  └──────────────────────┘        │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                     Persistence Layer                       │
│  ┌──────────────────────┐  ┌──────────────────────┐        │
│  │  WorkflowExecution   │  │  TaskExecution      │        │
│  │  - tasks: Vec        │  │  - status            │        │
│  │  - Order ❌          │  │  - workflow_id        │        │
│  └──────────────────────┘  └──────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         GUI Layer                             │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │  Details Page    │  │  History Page    │                 │
│  │  - Task List     │  │  - Execution List│                 │
│  │  ✅ Ordered      │  │                  │                 │
│  │  - From snapshot │  │                  │                 │
│  └──────────────────┘  └──────────────────┘                 │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                         Core Layer                           │
│  ┌──────────────────────┐  ┌──────────────────────┐        │
│  │  execution.rs       │  │  resume.rs           │        │
│  │  - Store snapshot  │  │  - Preserve snapshot │        │
│  │  - Pause detection │  │                      │        │
│  │  ✅ Fixed          │  │                      │        │
│  └──────────────────────┘  └──────────────────────┘        │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                        Engine Layer                          │
│  ┌──────────────────────┐  ┌──────────────────────┐        │
│  │  engine.rs          │  │  user_input.rs       │        │
│  │  - Execution loop    │  │  - Handler           │        │
│  │  ✅ Pause works     │  │  ✅ Returns waiting  │        │
│  └──────────────────────┘  └──────────────────────┘        │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                     Persistence Layer                       │
│  ┌──────────────────────────────────────────┐              │
│  │  WorkflowExecution                       │              │
│  │  - workflow_snapshot: serde_json::Value   │              │
│  │  - tasks: Vec (ordered)                 │              │
│  │  ✅ Self-contained                     │              │
│  └──────────────────────────────────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

### Workflow Execution Flow (Current)

```
1. User triggers workflow execution
2. Core loads WorkflowDefinition from database
3. Content parsed to EngineWorkflow
4. EngineWorkflow flattened during parsing
5. Tasks executed in flattened order
6. Results saved to WorkflowExecution
   ❌ Original structure lost
   ❌ Tasks in wrong order
```

### Workflow Execution Flow (Proposed)

```
1. User triggers workflow execution
2. Core loads WorkflowDefinition from database
3. Content parsed to EngineWorkflow
4. ✅ workflow_snapshot stored in WorkflowExecution
5. EngineWorkflow flattened for execution
6. Tasks executed
7. Results saved to WorkflowExecution
   ✅ Original structure preserved
   ✅ Tasks can be reordered correctly
```

### Task Ordering Flow

```
1. GUI loads WorkflowExecution
2. Parse workflow_snapshot JSON
3. Extract task IDs recursively (DFS from snapshot)
4. Create ordered task map
5. Filter tasks from execution by ordered IDs
6. Display in correct execution order
```

## Snapshot Design

### Data Structure

```rust
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub workflow_snapshot: serde_json::Value,  // NEW
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub tasks: Vec<TaskExecution>,  // Executed tasks
    pub timestamp: DateTime<Utc>,
    pub audit_trail: Vec<AuditEvent>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}
```

### Snapshot Content

The `workflow_snapshot` field contains the complete workflow JSON as it existed at execution time:

```json
{
  "id": "workflow-123",
  "name": "My Workflow",
  "tasks": [
    {
      "id": "task1",
      "name": "First Task",
      "function": {...},
      "next_tasks": [
        {
          "id": "task2",
          "name": "Second Task",
          "function": {...},
          "next_tasks": []
        }
      ]
    }
  ]
}
```

This preserves:
- Exact workflow structure
- Task execution order
- Nested task relationships
- Original JSON as submitted

## Bug Investigation Architecture

### Pause Mechanism Flow

```
1. Engine Loop
   ├─ Check ready tasks
   ├─ UserInputHandler.execute()
   │  ├─ Set status to WaitingForInput
   │  └─ Return { "waiting_for_input": true }
   ├─ Process result
   │  ├─ Detect waiting_for_input
   │  └─ Add to waiting_for_input set
   ├─ Check if waiting
   └─ ❌ LOOP SHOULD PAUSE HERE

2. Core API
   ├─ Check engine result tasks
   ├─ Look for WaitingForInput status
   ├─ Save execution as paused
   └─ ❌ NOT DETECTING CORRECTLY

3. GUI
   ├─ Load execution
   ├─ Check task statuses
   └─ Show "waiting for input"
```

### Debug Points

1. **UserInputHandler** (engine/src/handlers/user_input.rs)
   - Verify return value structure
   - Check TaskResult.output contains "waiting_for_input"
   - Confirm TaskStatus set to WaitingForInput

2. **Engine Loop** (engine/src/engine.rs lines 210-222)
   - Verify waiting_for_input detection
   - Check HashSet insertion
   - Confirm loop break condition

3. **Core API** (core/src/api/execution.rs lines 73-105)
   - Verify TaskStatus::WaitingForInput checking
   - Check result.task iteration
   - Confirm early return on pause

4. **Status Conversion** (core/src/bridge/task.rs)
   - Verify engine::TaskStatus → persistence::TaskStatus
   - Check WaitingForInput mapping
   - Confirm no data loss

## SRP Compliance

### File Organization

Following Single Responsibility Principle:

- **Models**: `persistence/src/models/execution.rs` - ONLY model definitions
- **Store**: `persistence/src/store/execution.rs` - ONLY CRUD operations
- **API**: `core/src/api/execution.rs` - ONLY execution logic
- **Bridge**: `core/src/bridge/execution.rs` - ONLY conversions
- **GUI**: `gui/src/pages/executions/details/` - ONLY UI components

### Test Organization

- `persistence/tests/models/execution_tests.rs` - Model tests
- `persistence/tests/store/execution_tests.rs` - Store tests
- `core/tests/api/execution_tests.rs` - API tests
- `core/tests/bridge/execution_tests.rs` - Bridge tests

Each test file tests ONLY one module.

## Benefits

### 1. Task Ordering
- Preserves exact workflow structure
- Enables correct task display order
- Self-contained execution records

### 2. Audit Trail
- Complete workflow snapshot stored
- Can replay exact execution
- Historical accuracy

### 3. Workflow Details
- Extract metadata from snapshot
- No external workflow lookups
- Survives workflow deletion

### 4. Bug Fix
- Proper workflow pause mechanism
- Correct user input handling
- Resume functionality works

## Trade-offs

### Storage
- **Pros**: Complete workflow preservation
- **Cons**: Larger database size (workflow JSON duplicated in each execution)

### Performance
- **Pros**: No external lookups needed
- **Cons**: JSON parsing required for ordering

### Migration
- **Pros**: Clean implementation
- **Cons**: Database reset required (no backward compatibility)

## Implementation Strategy

1. **Phase 1**: Add workflow_snapshot field (Persistence)
2. **Phase 2**: Store snapshot on execution (Core)
3. **Phase 3**: Implement ordering logic (GUI)
4. **Phase 4**: Debug and fix pause bug (Engine/Core)
5. **Phase 5**: Comprehensive testing
6. **Phase 6**: Final documentation and quality checks

## Conclusion

This architecture provides:
- ✅ Correct task ordering via workflow snapshot
- ✅ Self-contained execution records
- ✅ Complete audit trail
- ✅ Proper workflow pause mechanism
- ✅ Clean SRP-compliant implementation

