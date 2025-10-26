# Implementation Steps - User Input

## Overview

This document provides detailed step-by-step implementation instructions for adding user input support to workflows.

## Pre-Implementation Checklist

- [ ] Read all specification documents
- [ ] Understand architecture and data flow
- [ ] Set up development environment
- [ ] Review existing codebase patterns
- [ ] Verify database migrations work
- [ ] Run existing test suite (all passing)

## Phase 1: Persistence Layer

**Estimated Time**: 4-6 hours

### Step 1.1: Create UserInputRequest Model

**File**: `persistence/src/models/user_input_request.rs`

```bash
# Create file
touch persistence/src/models/user_input_request.rs
```

**Tasks**:
1. Create `UserInputRequest` struct with all fields
2. Create `InputType` enum (String, Number, Boolean)
3. Create `InputRequestStatus` enum (Pending, Fulfilled)
4. Implement Default
5. Implement validation methods
6. Add serialization/deserialization

**Validation**: 
- Write basic unit tests
- Test serialization round-trip

### Step 1.2: Extend TaskExecution Model

**File**: `persistence/src/models/task.rs`

**Tasks**:
1. Add `user_input: Option<String>` field
2. Add `input_request_id: Option<String>` field
3. Add `prompt_id: Option<String>` field
4. Update validation logic
5. Add helper methods (`has_user_input`, etc.)

**Validation**:
- Existing tests still pass
- New fields serialize correctly

### Step 1.3: Update Models Module

**File**: `persistence/src/models/mod.rs`

```rust
pub mod user_input_request; // NEW
```

**Validation**: Compiles successfully

### Step 1.4: Create Store Operations

**File**: `persistence/src/store/user_input.rs`

**Tasks**:
1. Implement `save_input_request()`
2. Implement `get_input_request()`
3. Implement `get_input_request_by_task()`
4. Implement `get_pending_inputs_for_workflow()`
5. Implement `fulfill_input_request()`
6. Implement `delete_input_request()`

**Validation**:
- Each method has logging
- Follows existing patterns

### Step 1.5: Extend Task Store Operations

**File**: `persistence/src/store/task.rs`

**Tasks**:
1. Implement `save_task_with_input()`
2. Implement `get_tasks_waiting_for_input()`
3. Implement `get_tasks_waiting_for_input_in_workflow()`
4. Extend existing methods to handle new fields

**Validation**: All methods work correctly

### Step 1.6: Update Store Module

**File**: `persistence/src/store/mod.rs`

```rust
pub mod user_input; // NEW
```

**Validation**: Compiles successfully

### Step 1.7: Add Database Table

**File**: `persistence/src/store/lib.rs`

**Tasks**:
1. Add `user_input_requests` table to `create_tables()`
2. Run migration (test in isolation)

**Validation**: 
- Table created successfully
- No data loss
- Backward compatible

### Step 1.8: Write Persistence Tests

**Files**: 
- `persistence/tests/models/user_input_request_tests.rs`
- `persistence/tests/models/task_input_tests.rs`
- `persistence/tests/store/user_input_tests.rs`

**Tasks**:
1. Test model creation
2. Test store operations
3. Test integration
4. Test error cases

**Validation**: All tests pass (>90% coverage)

## Phase 2: Engine Layer

**Estimated Time**: 6-8 hours

### Step 2.1: Extend TaskFunction Enum

**File**: `engine/src/types.rs`

**Tasks**:
1. Add `UserInput` variant to `TaskFunction`
2. Update serialization names
3. Update `get_function_type()` helper

**Validation**: Compiles, existing tests pass

### Step 2.2: Create UserInputHandler

**File**: `engine/src/handlers/user_input.rs`

**Tasks**:
1. Create `UserInputHandler` struct
2. Implement `TaskHandler` trait
3. Handle input configuration
4. Mark task as WaitingForInput
5. Return appropriate TaskResult
6. Add comprehensive logging

**Validation**:
- Handler executes without error
- Output format correct
- Status updates properly

### Step 2.3: Register Handler

**File**: `engine/src/handlers/mod.rs`

**Tasks**:
1. Add `pub mod user_input;`
2. Add registration in `HandlerRegistry::new()`
3. Export handler

**Validation**: Handler available in registry

### Step 2.4: Modify Engine Execution Loop

**File**: `engine/src/engine.rs`

**Tasks**:
1. Add `waiting_for_input` HashSet to execution state
2. Modify `get_ready_tasks_from_tree()` to skip waiting tasks
3. Detect WaitingForInput in task results
4. Add tasks to waiting set
5. Handle workflow pause/resume

**Validation**:
- Engine pauses when input needed
- Parallel tasks continue
- Resume works correctly

### Step 2.5: Add Resume Method

**File**: `engine/src/engine.rs`

**Tasks**:
1. Implement `resume_task_with_input()`
2. Handle input validation
3. Continue task execution
4. Execute next_tasks

**Validation**: Resume works end-to-end

### Step 2.6: Create Example Workflows

**Files**:
- `engine/examples/user_input_simple.json`
- `engine/examples/user_input_parallel.json`
- `engine/examples/user_input_nested.json`

**Tasks**:
1. Create simple input workflow
2. Create parallel input workflow
3. Create nested input workflow

**Validation**: Workflows parse correctly

### Step 2.7: Write Engine Tests

**Files**:
- `engine/tests/user_input_handler_tests.rs`
- `engine/tests/resume_tests.rs`
- `engine/tests/engine_user_input_tests.rs`

**Tasks**:
1. Test handler
2. Test engine modifications
3. Test full workflows
4. Test parallel scenarios

**Validation**: All tests pass (>90% coverage)

## Phase 3: Core Bridge & API

**Estimated Time**: 4-6 hours

### Step 3.1: Create Bridge Conversions

**File**: `core/src/bridge/user_input.rs`

**Tasks**:
1. Convert engine → persistence types
2. Convert persistence → engine types
3. Handle enum conversions
4. Add error handling

**Validation**: Round-trip conversions work

### Step 3.2: Update Bridge Module

**File**: `core/src/bridge/mod.rs`

```rust
pub mod user_input; // NEW
```

**Validation**: Compiles

### Step 3.3: Create Input Management API

**File**: `core/src/api/input.rs`

**Tasks**:
1. Implement `provide_user_input()`
2. Implement `get_pending_inputs()`
3. Implement `get_tasks_waiting_for_input()`
4. Add input validation
5. Add comprehensive error handling
6. Add logging

**Validation**: 
- API methods work
- Error handling correct
- Logging comprehensive

### Step 3.4: Update API Module

**File**: `core/src/api/mod.rs`

```rust
pub mod input; // NEW
```

**Validation**: Exports correctly

### Step 3.5: Enhance Execution API

**File**: `core/src/api/execution.rs`

**Tasks**:
1. Detect WaitingForInput status
2. Update workflow state
3. Handle paused workflows

**Validation**: Paused workflows handled correctly

### Step 3.6: Enhance Resume API

**File**: `core/src/api/resume.rs`

**Tasks**:
1. Check for input requirement
2. Validate input present
3. Integrate with provide_user_input

**Validation**: Resume with input works

### Step 3.7: Extend Error Types

**File**: `core/src/errors.rs`

**Tasks**:
1. Add input-related error variants
2. Add error messages
3. Update error handling

**Validation**: Error handling comprehensive

### Step 3.8: Write Core Tests

**Files**:
- `core/tests/api/input_tests.rs`
- `core/tests/bridge/user_input_tests.rs`
- `core/tests/integration/user_input_tests.rs`

**Tasks**:
1. Test API methods
2. Test bridge conversions
3. Test integration

**Validation**: All tests pass (>90% coverage)

## Phase 4: CLI Integration

**Estimated Time**: 3-4 hours

### Step 4.1: Create Commands Module

**File**: `cli/src/commands/mod.rs`

**Tasks**:
1. Create module structure
2. Export commands

**Validation**: Compiles

### Step 4.2: Implement List Command

**File**: `cli/src/commands/list.rs`

**Tasks**:
1. List all workflows
2. Format output nicely
3. Handle empty list

**Validation**: Lists workflows correctly

### Step 4.3: Implement Start Command

**File**: `cli/src/commands/start.rs`

**Tasks**:
1. Start workflow execution
2. Handle interactive input
3. Show progress
4. Handle pause/resume

**Validation**: Start works with input

### Step 4.4: Implement Resume Command

**File**: `cli/src/commands/resume.rs`

**Tasks**:
1. Get pending inputs
2. Prompt for each
3. Validate input
4. Submit and resume

**Validation**: Resume works correctly

### Step 4.5: Create Input Modules

**Files**:
- `cli/src/input/mod.rs`
- `cli/src/input/prompt.rs`
- `cli/src/input/validator.rs`

**Tasks**:
1. Implement prompt display
2. Implement input gathering
3. Implement validation
4. Add retry logic

**Validation**: Input works correctly

### Step 4.6: Update Main

**File**: `cli/src/main.rs`

**Tasks**:
1. Add subcommands
2. Wire up commands
3. Handle command execution

**Validation**: Commands work from CLI

### Step 4.7: Write CLI Tests

**File**: `cli/tests/cli_input_tests.rs`

**Tasks**:
1. Test each command
2. Test input validation
3. Test error handling

**Validation**: Tests pass

## Phase 5: GUI Components

**Estimated Time**: 4-5 hours

### Step 5.1: Create UserInputForm Component

**File**: `gui/src/components/forms/user_input_form.rs`

**Tasks**:
1. Create form structure
2. Add input field
3. Add validation
4. Add submit button
5. Handle API call
6. Show loading/error states

**Validation**: Form works correctly

### Step 5.2: Update Forms Module

**File**: `gui/src/components/forms/mod.rs`

```rust
pub mod user_input_form; // NEW
```

**Validation**: Exports correctly

### Step 5.3: Modify Task Details Panel

**File**: `gui/src/pages/executions/details/components/task_details_panel.rs`

**Tasks**:
1. Add input form section
2. Show when WaitingForInput
3. Add "Input Needed" badge
4. Style amber/yellow

**Validation**: UI shows correctly

### Step 5.4: Update Workflow Flow Visual

**File**: `gui/src/pages/executions/details/components/workflow_flow.rs`

**Tasks**:
1. Add amber colors for WaitingForInput
2. Add pulsing animation
3. Update node styling

**Validation**: Visual indicators work

### Step 5.5: Update Execution Details Page

**File**: `gui/src/pages/executions/details/page.rs`

**Tasks**:
1. Show pending input count
2. Filter tasks by status

**Validation**: Count and filter work

### Step 5.6: Write GUI Tests

**File**: `gui/tests/components/user_input_form_tests.rs`

**Tasks**:
1. Test component render
2. Test input handling
3. Test API integration

**Validation**: Tests pass

## Phase 6: Integration Testing

**Estimated Time**: 3-4 hours

### Step 6.1: End-to-End Tests

**Tasks**:
1. Create full workflow test
2. Test pause/resume cycle
3. Test parallel scenarios
4. Test error cases

**Validation**: All scenarios work

### Step 6.2: Performance Tests

**Tasks**:
1. Test input creation speed
2. Test validation speed
3. Test resume speed

**Validation**: Performance acceptable

### Step 6.3: Concurrency Tests

**Tasks**:
1. Test multiple simultaneous inputs
2. Test race conditions
3. Test data consistency

**Validation**: No race conditions

## Phase 7: Documentation

**Estimated Time**: 2-3 hours

### Step 7.1: Update README

**Tasks**:
1. Add user input examples
2. Document commands
3. Document GUI features

### Step 7.2: Create Example Workflows

**Tasks**:
1. Create example files
2. Document each example
3. Add to repository

### Step 7.3: API Documentation

**Tasks**:
1. Document new API methods
2. Add usage examples
3. Document error cases

## Post-Implementation Checklist

- [ ] All tests pass
- [ ] No compiler warnings
- [ ] No linter errors
- [ ] Code formatted
- [ ] Documentation complete
- [ ] Examples work
- [ ] Performance acceptable
- [ ] Security reviewed

## Rollback Procedures

If critical issues found:

1. **Documentation Phase**: Roll back document changes
2. **Implementation Phase**: Revert commits, fix issues, recommit
3. **Testing Phase**: Fix tests, update implementation if needed

## Validation Checkpoints

After each phase:
- [ ] Code compiles
- [ ] Tests pass
- [ ] No regressions
- [ ] Logging works
- [ ] Error handling works

## Common Issues & Solutions

### Issue: Database Migration Fails

**Solution**: Check table exists, verify schema, test in isolation

### Issue: Handler Not Found

**Solution**: Verify registration in HandlerRegistry

### Issue: Input Not Validating

**Solution**: Check input type mapping, verify validator logic

### Issue: GUI Not Updating

**Solution**: Check state management, verify API calls

## Success Criteria

- ✅ All workflows with input work correctly
- ✅ CLI shows prompts and accepts input
- ✅ GUI displays input forms
- ✅ Parallel tasks handled independently
- ✅ Resume works correctly
- ✅ All tests pass
- ✅ No regressions in existing features

