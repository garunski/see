# Original Feature Plan

This is the original plan that was provided for the user input pause feature implementation.

<!-- 986fefa2-f166-48e5-8503-24483c9a80f9 a9cc8d3e-431e-4590-8363-9b731a7f96aa -->
# User Input Pause Feature Implementation Plan

## Overview

Add support for pausing workflow execution to wait for user input (yes/no button). The feature supports:

- Task definition configuration + runtime dynamic prompts
- Multiple sequential prompts per task
- Persistence across app restarts (user must navigate to execution details to resume)
- No timeout (wait indefinitely)
- User response passed to task handler to influence behavior

## Architecture Changes

### 1. Core Type System Updates

**File: `core/src/types.rs`**

- Add `WaitingForInput` variant to `TaskStatus` enum
- Add `Cancelled` variant to `TaskStatus` for when user clicks "No"
- Create new `UserInputRequest` struct:
  ```rust
  pub struct UserInputRequest {
      pub prompt_text: String,
      pub prompt_id: String,  // unique ID for this specific prompt
      pub created_at: String,
  }
  ```

- Add `user_input_requests: Vec<UserInputRequest>` to `TaskInfo`
- Add `user_responses: HashMap<String, bool>` to `TaskInfo` (prompt_id -> yes/no)

**File: `core/src/persistence/models.rs`**

- Add `WaitingForInput` to `WorkflowStatus` enum
- Create `PendingUserInput` struct for persistence:
  ```rust
  pub struct PendingUserInput {
      pub execution_id: String,
      pub task_id: String,
      pub prompt_text: String,
      pub prompt_id: String,
      pub created_at: String,
  }
  ```

- Add `pending_inputs: Vec<UserInputRequest>` field to `TaskExecution`

### 2. Persistence Layer Updates

**File: `core/src/persistence/store.rs`**

- Add new table `PENDING_INPUTS_TABLE` for storing pending user inputs
- Add `save_pending_input()` method to `AuditStore` trait
- Add `get_pending_inputs(execution_id, task_id)` method
- Add `delete_pending_input(execution_id, task_id, prompt_id)` method
- Add `list_workflows_waiting_for_input()` method to find all paused workflows
- Update `save_task_execution()` to persist pending inputs field

### 3. Execution Context Updates

**File: `core/src/execution/context.rs`**

- Add `request_user_input(task_id, prompt_text, prompt_id)` method
- Add `provide_user_response(task_id, prompt_id, response: bool)` method
- Add `get_pending_inputs(task_id)` method
- Add `get_user_response(task_id, prompt_id) -> Option<bool>` method
- Update task status handling to support `WaitingForInput` and `Cancelled`

### 4. Task Execution Coordination

**File: `core/src/engine/execute.rs`**

- Modify `execute_workflow_from_content()` to check for existing pending inputs on start
- If pending inputs exist, mark workflow as `WaitingForInput` and return early
- Add new `resume_workflow_execution(execution_id)` function:
  - Load workflow metadata and task state from DB
  - Reconstruct execution context with existing logs/state
  - Check which task was waiting for input
  - Continue execution from that task with user responses
- Add `provide_user_input_and_resume(execution_id, task_id, prompt_id, response: bool)` function

### 5. Handler Updates for User Input

**File: `core/src/task_executor.rs`**

- Add `request_user_input(&self, prompt_text: &str, prompt_id: &str)` to `TaskLogger` trait
- Add `get_user_response(&self, prompt_id: &str) -> Option<bool>` to `TaskLogger` trait
- Implement these methods in `ContextTaskLogger`

**Files: `core/src/engine/handlers/cli_command.rs` and `cursor_agent.rs`**

- Check task config for `"requires_user_input"` array field:
  ```json
  "requires_user_input": [
    {"prompt_id": "confirm-start", "prompt": "Run this command?", "when": "before"},
    {"prompt_id": "confirm-result", "prompt": "Accept result?", "when": "after"}
  ]
  ```

- Before execution: check for "before" prompts
  - If user response not yet available, request input and return `WaitingForInput` error
  - If response is false, return `Cancelled` error
- After execution: check for "after" prompts (similar logic)
- Runtime dynamic prompts: handlers can call `logger.request_user_input()` during execution

### 6. Error Handling for Paused State

**File: `core/src/errors.rs`**

- Add `WaitingForUserInput` error variant with metadata:
  ```rust
  WaitingForUserInput {
      task_id: String,
      prompt_text: String,
      prompt_id: String,
  }
  ```

- Add `UserCancelled` error variant

**Update handlers to return this error when input needed**

### 7. GUI State Management

**File: `gui/src/state/history_state.rs`**

- Add `waiting_for_input_workflows: Vec<WorkflowMetadata>` field
- Add methods to manage waiting workflows

**New File: `gui/src/state/user_input_state.rs`**

- Create state for managing user input UI:
  ```rust
  pub struct UserInputState {
      pub pending_prompts: HashMap<String, Vec<PendingUserInput>>, // execution_id -> prompts
      pub responding_to: Option<(String, String, String)>, // (execution_id, task_id, prompt_id)
  }
  ```


### 8. GUI Service Layer

**New File: `gui/src/services/user_input.rs`**

- `get_pending_inputs(execution_id: String, task_id: String)`
- `submit_user_response(execution_id: String, task_id: String, prompt_id: String, response: bool)`
- `list_waiting_executions()` - fetch all executions with pending inputs

**File: `gui/src/services/history.rs`**

- Update to load workflows with `WaitingForInput` status

### 9. GUI Execution Details Page

**File: `gui/src/pages/executions/details/page.rs`**

- Add detection for tasks in `WaitingForInput` status
- Show user input prompt UI when task is waiting

**New File: `gui/src/pages/executions/details/components/user_input_panel.rs`**

- Create UI component with:
  - Prompt text display
  - Large "Yes" and "No" buttons
  - Timestamp when input was requested
  - Warning that this affects workflow execution
- On button click:
  - Call service to submit response
  - Trigger workflow resumption
  - Show loading state while resuming
  - Refresh execution details

**File: `gui/src/pages/executions/details/components/task_details_panel.rs`**

- Update to show "Waiting for User Input" badge for tasks with pending prompts
- Show history of previous prompts and responses

### 10. Workflow Status Indicators

**File: `gui/src/pages/executions/history/page.rs`**

- Add visual indicator for workflows with `WaitingForInput` status
- Add filter to show only waiting workflows
- Add "Resume" button/link to navigate to execution details

**File: `gui/src/pages/executions/details/components/execution_overview.rs`**

- Show "Waiting for Input" status badge
- Display which task is waiting

### 11. Startup Recovery

**File: `gui/src/main.rs` or app initialization**

- On app startup, query for workflows with `WaitingForInput` status
- Update history state with these workflows
- No automatic resumption - user must navigate to execution details

## Implementation Order

1. Core type system updates (TaskStatus, UserInputRequest)
2. Persistence layer (new table, store methods)
3. Execution context (input request/response methods)
4. Error handling for paused state
5. Handler updates (check for required input, pause execution)
6. Resume workflow execution logic
7. GUI state management
8. GUI service layer
9. User input panel component
10. Execution details page integration
11. History page indicators
12. Startup recovery

## Key Technical Challenges

1. **State reconstruction after restart**: Must rebuild execution context from persisted state
2. **Task handler coordination**: Handlers must check for responses before/after operations
3. **Async flow interruption**: Using custom error to signal pause, not true async await
4. **Multiple sequential prompts**: Track which prompt in sequence is active
5. **Database consistency**: Ensure task state and pending inputs stay in sync

## Logging and Tracing Strategy

### Core Layer Tracing Points

**`core/src/execution/context.rs`**

- `request_user_input()`: Log prompt_id, task_id, prompt_text, timestamp
- `provide_user_response()`: Log prompt_id, task_id, response (yes/no), timestamp
- `get_user_response()`: Log lookup attempts and results
- Status transitions to/from `WaitingForInput`: Log old status -> new status with task context

**`core/src/persistence/store.rs`**

- `save_pending_input()`: Log execution_id, task_id, prompt_id, success/failure
- `get_pending_inputs()`: Log execution_id, task_id, count of inputs found
- `delete_pending_input()`: Log execution_id, task_id, prompt_id, success/failure
- `list_workflows_waiting_for_input()`: Log count of waiting workflows found
- All DB operations: Use `#[instrument]` attribute with relevant fields

**`core/src/engine/execute.rs`**

- `execute_workflow_from_content()`: Check for pending inputs at start, log if found
- `resume_workflow_execution()`: Log execution_id, which task resuming from, pending inputs count
- `provide_user_input_and_resume()`: Log full flow: input received -> task resumption -> completion
- Workflow state transitions: Running -> WaitingForInput -> Running (resumed)
- Task skipping logic: Log which tasks already completed vs. which to resume

**`core/src/engine/handlers/cli_command.rs` and `cursor_agent.rs`**

- Before checking for required input: Log task_id, config inspection
- When input required but not available: Log prompt_id, prompt_text, pausing execution
- When input received and validated: Log prompt_id, response, continuing execution
- When user cancels (response=false): Log prompt_id, task_id, cancellation reason
- Dynamic runtime input requests: Log why input is needed, what triggered it

**`core/src/errors.rs`**

- `WaitingForUserInput` error: Include execution_id, task_id, prompt_id in debug output
- `UserCancelled` error: Include full context of what was cancelled

### GUI Layer Tracing Points

**`gui/src/services/user_input.rs`**

- `get_pending_inputs()`: Log execution_id, task_id, inputs retrieved
- `submit_user_response()`: Log execution_id, task_id, prompt_id, response, API call status
- `list_waiting_executions()`: Log count found, execution_ids

**`gui/src/services/history.rs`**

- Loading workflows with `WaitingForInput` status: Log count, execution_ids
- State updates: Log before/after state

**`gui/src/pages/executions/details/page.rs`**

- Component mount: Log execution_id, check for waiting tasks
- Detecting waiting state: Log task_id, status, pending prompts
- UI state changes: Log when user input panel shows/hides

**`gui/src/pages/executions/details/components/user_input_panel.rs`**

- Component render: Log execution_id, task_id, prompt details
- Button click: Log which button (yes/no), prompt_id, timestamp
- Response submission: Log API call start, success, failure
- Workflow resumption trigger: Log execution_id, expected next state

**App Startup (`gui/src/main.rs`)**

- Query for waiting workflows: Log count found
- State initialization: Log waiting workflows loaded into state

### Tracing Guidelines

1. **Use structured logging with tracing crate:**
   ```rust
   #[instrument(skip(self), fields(execution_id, task_id, prompt_id))]
   ```

2. **Log levels:**

   - `trace`: Fine-grained flow (loop iterations, checks)
   - `debug`: Function entry/exit, decision points
   - `info`: State transitions, user actions, workflow lifecycle events
   - `warn`: Unexpected but recoverable situations
   - `error`: Failures, errors that stop execution

3. **Span context:**

   - Create spans for multi-step operations (resume workflow, handle user input)
   - Include execution_id in all relevant spans
   - Use `Span::current().record()` to add fields dynamically

4. **Database operations:**

   - Log before write operations: what will be saved
   - Log after write operations: success/failure, row counts
   - Log query parameters for all reads
   - Log empty result sets (important for debugging missing data)

5. **State transitions:**

   - Always log old_state -> new_state with full context
   - Include timestamp in state transition logs
   - Log what triggered the transition

6. **Error paths:**

   - Log full error context before propagating
   - Include recovery attempts in logs
   - Log user-facing error messages separately from internal errors

7. **Resume logic:**

   - Log entire state reconstruction process
   - Log which tasks are being skipped (already complete)
   - Log from which task execution resumes
   - Log verification that state matches expected

## Files to Create

- `gui/src/services/user_input.rs`
- `gui/src/state/user_input_state.rs`
- `gui/src/pages/executions/details/components/user_input_panel.rs`

## Files to Modify

- `core/src/types.rs`
- `core/src/persistence/models.rs`
- `core/src/persistence/store.rs`
- `core/src/execution/context.rs`
- `core/src/task_executor.rs`
- `core/src/engine/execute.rs`
- `core/src/engine/handlers/cli_command.rs`
- `core/src/engine/handlers/cursor_agent.rs`
- `core/src/errors.rs`
- `gui/src/state/history_state.rs`
- `gui/src/services/history.rs`
- `gui/src/pages/executions/details/page.rs`
- `gui/src/pages/executions/details/components/task_details_panel.rs`
- `gui/src/pages/executions/details/components/execution_overview.rs`
- `gui/src/pages/executions/history/page.rs`

### To-dos

- [x] Add WaitingForInput/Cancelled to TaskStatus, create UserInputRequest struct
- [x] Add WaitingForInput to WorkflowStatus, create PendingUserInput struct
- [x] Add pending inputs table and CRUD methods to RedbStore
- [x] Add input request/response methods to ExecutionContext
- [x] Add WaitingForUserInput and UserCancelled error variants
- [x] Extend TaskLogger trait with user input methods
- [x] Update cli_command and cursor_agent handlers to check for required input
- [x] Implement resume_workflow_execution and provide_user_input_and_resume functions
- [x] Create user_input_state.rs and update history_state.rs
- [x] Create user_input service and update history service
- [x] Create user input panel component with Yes/No buttons
- [x] Integrate user input panel into execution details page
- [x] Add waiting status indicators and filters to history page
- [x] Load waiting workflows on app startup

**Note**: All to-dos were marked as completed, but the implementation failed due to critical runtime issues.
