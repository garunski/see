# User Input Workflows - Architecture

## Overview

This document describes the architecture for implementing pausable/resumable workflows with user input support. The feature enables workflows to pause execution at specific tasks that require user-provided data, allowing users to provide input and automatically resume execution.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          User Interface Layer                    │
├──────────────────────────┬──────────────────────────────────────┤
│ GUI                     │ CLI                                 │
│ - Input forms           │ - Interactive prompts                 │
│ - Visual indicators     │ - Progress display                    │
│ - Resume actions        │ - Input validation                    │
└────────┬─────────────────┴────┬──────────────────────────────────┘
         │                      │
         └──────────────────────┼──────────────────────────────┐
                                │                             │
                    ┌───────────▼─────────────┐   ┌──────────▼───────────┐
                    │   Core API Layer         │   │   Persistence       │
                    │   - Input management     │   │   - UserInputRequest│
                    │   - Resume logic         │   │   - TaskExecution   │
                    │   - State management     │   │   - WorkflowExecution│
                    └───────────┬──────────────┘   └──────────┬──────────┘
                                │                             │
                    ┌───────────▼──────────────┐              │
                    │   Engine Layer           │              │
                    │   - UserInput handler    │              │
                    │   - Pause/resume logic   │              │
                    │   - Parallel execution   │              │
                    └──────────────────────────┘              │
                                                              │
                    ┌─────────────────────────────────────────┘
                    │
                    ▼
            SQLite Database
```

## Data Flow

### 1. Workflow Execution Starts

```
WorkflowExecution (Status: Running)
├── TaskExecution 1 (Status: Complete)
├── TaskExecution 2 (Status: WaitingForInput) ← USER INPUT REQUIRED
├── TaskExecution 3 (Status: InProgress)
└── TaskExecution 4 (Status: Pending)

↓ Engine creates UserInputRequest
↓ Stores in database
↓ Returns control to caller
```

### 2. User Provides Input

```
User provides input → Core API
├── Validates input type
├── Updates TaskExecution.user_input
├── Marks input_request as fulfilled
└── Calls engine.resume_task_with_input()

↓ Engine resumes execution
↓ Continues with next_tasks
↓ Workflow progresses to completion
```

### 3. Parallel Task Handling

```
Parallel Group:
├── Task A: Complete
├── Task B: WaitingForInput ← User input provided
│   └── Resumes automatically, continues
├── Task C: InProgress
└── Task D: WaitingForInput ← Still waiting

Group completes when ALL tasks finish
```

## State Transitions

### TaskExecution Status Lifecycle

```
Pending
  ↓
InProgress
  ↓
   ├─→ Complete (normal flow)
   │
   └─→ WaitingForInput (input required)
         ↓
         [User provides input]
         ↓
         InProgress (resumed)
         ↓
         Complete
```

### WorkflowExecution Status Lifecycle

```
Pending
  ↓
Running (workflow started)
  │
  ├─→ Complete (all tasks done)
  │
  ├─→ Running (some tasks waiting for input)
  │     │
  │     └─→ User provides all inputs
  │           │
  │           └─→ Complete
  │
  └─→ Failed (task failures)
```

## Crate Responsibilities

### Persistence (`persistence/`)

**Models:**
- `UserInputRequest` - Runtime input request tracking
- `TaskExecution` - Extended with input fields
- `WorkflowExecution` - Execution state

**Operations:**
- CRUD for input requests
- Task input operations
- Query operations for pending inputs

### Engine (`engine/`)

**Components:**
- `UserInputHandler` - Executes input tasks
- `WorkflowEngine` - Enhanced with pause/resume logic
- Task execution loop - Handles WaitingForInput state

**Behavior:**
- Creates input requests when encountering user input task
- Does NOT block execution (returns immediately)
- Supports resume with input data

### Core (`core/`)

**Bridge Layer:**
- Conversions between engine/persistence types
- Input request transformations
- Task state conversions

**API Layer:**
- `provide_user_input()` - Submit input and resume
- `get_pending_inputs()` - List tasks waiting for input
- Enhanced `execute_workflow()` - Handles paused workflows
- Enhanced `resume_task()` - Validates and applies input

### CLI (`cli/`)

**Commands:**
- `workflow list` - List available workflows
- `workflow start <id>` - Start workflow with interactive input
- `workflow resume <execution_id>` - Resume paused workflow

**Interaction:**
- Prompts user when input required during execution
- Validates input before submission
- Shows progress and status

### GUI (`gui/`)

**Components:**
- Input form in task details slideout
- Visual indicators (amber styling, badges)
- Resume button with input submission

**Features:**
- Real-time status updates
- Input type-specific forms
- Validation feedback

## Key Design Decisions

### 1. Separation of Concerns

**Why**: Follow SRP - one model/operation per concern

**Implementation**:
- `UserInputRequest` - Runtime state (separate from templates)
- `user_prompts` - Template library (existing, unchanged)
- `TaskExecution.user_input` - Actual input value
- `TaskExecution.input_request_id` - Link to request

### 2. Non-Blocking Input Tasks

**Why**: Don't block execution loop - allow other tasks to continue

**Implementation**:
- UserInput handler returns immediately
- Marks task as WaitingForInput
- Engine continues with other ready tasks
- Resume separately when input provided

### 3. Independent Parallel Resume

**Why**: Each task resumes independently when its input arrives

**Implementation**:
- Parallel group tasks don't block each other
- Non-input tasks continue normally
- Input tasks wait independently
- Workflow completes when ALL tasks done

### 4. Input Type Safety

**Why**: Validate inputs to prevent errors downstream

**Implementation**:
- Store input_type in UserInputRequest
- Validate on input submission
- Support: string, number, boolean

### 5. Auto-Resume on Input

**Why**: Seamless UX - no explicit "resume" needed after input

**Implementation**:
- provide_user_input() automatically calls resume
- Validation happens before resume
- Errors shown without resume attempt

## Database Schema

### New Table: `user_input_requests`

```sql
CREATE TABLE IF NOT EXISTS user_input_requests (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
)
```

**JSON Structure:**
```json
{
    "id": "uuid",
    "task_execution_id": "task-123",
    "workflow_execution_id": "workflow-456",
    "prompt_text": "Enter your name",
    "input_type": "string",
    "required": true,
    "default_value": null,
    "validation_rules": {},
    "status": "pending|fulfilled",
    "created_at": "2024-01-01T00:00:00Z",
    "fulfilled_at": null
}
```

### Modified Table: `task_executions`

**New Fields:**
```json
{
    "user_input": "actual value provided",
    "input_request_id": "request-uuid",
    "prompt_id": "optional reference to user_prompts"
}
```

## Error Handling

### Invalid Input Types

- Reject and show error message
- Do NOT resume workflow
- Allow user to retry

### Missing Input Request

- Return error if request not found
- Log for debugging

### Resume Failures

- Log error with details
- Do NOT update task status
- Show error to user

### Concurrent Input

- Last write wins (simple approach)
- Validate status before accepting input
- Log conflicts for analysis

## Logging Strategy

### State Transitions
```
TRACE: Task {id} transitioning from InProgress to WaitingForInput
DEBUG: UserInputRequest created for task {id}
INFO: User provided input for task {id}
TRACE: Task {id} resuming from WaitingForInput to InProgress
```

### Input Operations
```
DEBUG: Validating input for task {id}, type={type}
TRACE: Input validation passed for task {id}
WARN: Input validation failed for task {id}: {error}
ERROR: Failed to resume task {id}: {error}
```

### Parallel Execution
```
DEBUG: Found {n} tasks waiting for input in parallel group
TRACE: Task {id} in parallel group received input
INFO: Resuming task {id} independently
```

## Performance Considerations

### Efficiency
- Input requests don't block execution
- Minimal database queries (batch where possible)
- Efficient state tracking

### Scalability
- Support hundreds of concurrent workflows
- Index on user_input_requests.task_execution_id
- Cleanup old input requests (future enhancement)

### Reliability
- Transactions for input submission
- Atomic state updates
- Retry logic for transient failures

## Testing Strategy

### Unit Tests
- Each layer tested independently
- Mock dependencies
- Test happy path and error cases

### Integration Tests
- Test full workflows with input
- Parallel scenarios
- Error conditions

### End-to-End Tests
- Real workflows with user input
- CLI and GUI paths
- Multiple input scenarios

## Future Enhancements

### Phase 2 Features
- Input templates with validation rules
- Timeout for input requests
- Input history tracking
- Batch input for multiple pending tasks

### Phase 3 Features
- Conditional inputs (if/then logic)
- Input suggestions/autocomplete
- Rich input types (files, dates, etc.)
- Input approval workflows

