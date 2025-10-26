# Testing Strategy - User Input

## Overview

This document outlines the comprehensive testing strategy for user input functionality across all crates.

## Testing Principles

### Universal Standards

1. **ALL tests in `/tests` directories** - Never in implementation files
2. **Deterministic tests** - No race conditions or timing dependencies
3. **Isolated tests** - Each test cleans up its resources
4. **Comprehensive coverage** - Happy path, edge cases, error conditions
5. **Fast execution** - Unit tests complete in seconds

### Test Organization

```
<crate>/
├── src/
│   ├── lib.rs
│   ├── <module>.rs
│   └── ...
└── tests/
    ├── <module>_tests.rs      # Unit tests for module
    ├── integration_tests.rs    # Integration tests
    ├── concurrency_tests.rs   # Concurrency/scaling tests
    └── logging_tests.rs        # Logging verification
```

## Persistence Layer Tests

### Test Files

1. **`persistence/tests/models/user_input_request_tests.rs`**
2. **`persistence/tests/models/task_input_tests.rs`**
3. **`persistence/tests/store/user_input_tests.rs`**
4. **`persistence/tests/task_input_operations_tests.rs`**
5. **`persistence/tests/integration/user_input_tests.rs`**

### Model Tests

**UserInputRequest**:
- ✅ Model creation with all fields
- ✅ Model validation (required fields, format checks)
- ✅ Serialization/deserialization round-trip
- ✅ Status transitions (Pending → Fulfilled)
- ✅ Error cases (empty IDs, invalid types)

**TaskExecution with Input**:
- ✅ Task with user_input field
- ✅ Task with input_request_id field
- ✅ Task with prompt_id field
- ✅ Validation with input fields
- ✅ Edge cases (None values, empty strings)

### Store Tests

**UserInputRequest Operations**:
- ✅ `save_input_request()` - create
- ✅ `get_input_request()` - read by ID
- ✅ `get_input_request_by_task()` - read by task
- ✅ `get_pending_inputs_for_workflow()` - batch query
- ✅ `fulfill_input_request()` - update status
- ✅ `delete_input_request()` - delete
- ✅ Error cases (not found, invalid data)

**Task Input Operations**:
- ✅ `save_task_with_input()` - save with input
- ✅ `get_tasks_waiting_for_input()` - query waiting tasks
- ✅ `get_tasks_waiting_for_input_in_workflow()` - filtered query
- ✅ Update task status with input
- ✅ Concurrent access handling

### Integration Tests

**UserInputRequest + TaskExecution**:
- Create input request
- Link to task execution
- Fulfill request and update task
- Verify data consistency

## Engine Layer Tests

### Test Files

1. **`engine/tests/user_input_handler_tests.rs`**
2. **`engine/tests/resume_tests.rs`**
3. **`engine/tests/engine_user_input_tests.rs`**
4. **`engine/tests/integration/user_input_tests.rs`**

### Handler Tests

**UserInputHandler**:
- ✅ Handler registration
- ✅ Handler execution
- ✅ Returns WaitingForInput status
- ✅ Output format validation
- ✅ Context updates
- ✅ Error handling

### Engine Tests

**Execute Workflow with Input**:
- ✅ Workflow with user input task
- ✅ Workflow pauses at input task
- ✅ Parallel tasks continue during wait
- ✅ Workflow resumes with input
- ✅ Next tasks execute after resume

**Parallel Input Scenarios**:
- ✅ Multiple tasks waiting for input in parallel
- ✅ Independent resume for each task
- ✅ Non-input tasks continue
- ✅ Workflow completes when all done

**Edge Cases**:
- ✅ Nested input (input → task → input)
- ✅ Input task at end of workflow
- ✅ Input task with no next_tasks
- ✅ Multiple sequential inputs

## Core Layer Tests

### Test Files

1. **`core/tests/api/input_tests.rs`**
2. **`core/tests/bridge/user_input_tests.rs`**
3. **`core/tests/integration/user_input_tests.rs`**

### API Tests

**Input Management API**:
- ✅ `provide_user_input()` - happy path
- ✅ Input validation (string, number, boolean)
- ✅ Task status updates
- ✅ Workflow resume
- ✅ Error cases (task not found, wrong status, invalid input)

**Get Operations**:
- ✅ `get_pending_inputs()` - list inputs
- ✅ `get_tasks_waiting_for_input()` - list tasks
- ✅ Empty results handling
- ✅ Error cases

### Bridge Tests

**User Input Conversions**:
- ✅ Engine → Persistence conversion
- ✅ Persistence → Engine conversion
- ✅ Type mapping (InputType enum)
- ✅ Status mapping (InputRequestStatus enum)
- ✅ Round-trip consistency

## CLI Layer Tests

### Test Files

1. **`cli/tests/cli_input_tests.rs`**
2. **`cli/tests/integration/user_input_tests.rs`**

### Command Tests

**List Command**:
- ✅ Lists all workflows
- ✅ Empty list handling
- ✅ Output formatting

**Start Command**:
- ✅ Starts workflow
- ✅ Interactive input prompts
- ✅ Input validation in CLI
- ✅ Workflow pause/resume
- ✅ Progress display

**Resume Command**:
- ✅ Lists pending inputs
- ✅ Prompts for each input
- ✅ Validates input
- ✅ Resumes workflow
- ✅ Error handling

## GUI Layer Tests

### Test Files

1. **`gui/tests/components/user_input_form_tests.rs`**
2. **`gui/tests/integration/user_input_tests.rs`**

### Component Tests

**UserInputForm**:
- ✅ Component renders
- ✅ Input field updates
- ✅ Validation feedback
- ✅ Submit button triggers API
- ✅ Loading state during submit
- ✅ Error message display

**Visual Indicators**:
- ✅ Status badge shows correctly
- ✅ Task box styling
- ✅ Amber color scheme
- ✅ Pulsing animation

## Integration Tests

### End-to-End Scenarios

**File**: Each crate's `tests/integration/user_input_e2e_tests.rs`

**Scenario 1: Simple Input Workflow**
```
1. Create workflow with user input task
2. Start workflow execution
3. Task reaches input point
4. Provide input via CLI/GUI
5. Verify task resumes
6. Verify workflow completes
```

**Scenario 2: Parallel Input Workflow**
```
1. Create workflow with parallel input tasks
2. Start execution
3. Both tasks pause for input
4. Provide input for first task → resumes
5. Provide input for second task → resumes
6. Verify both complete independently
7. Verify workflow completes
```

**Scenario 3: Nested Input Workflow**
```
1. Create workflow: task → input → next task → input
2. Start execution
3. Provide first input
4. Execution continues to second input
5. Provide second input
6. Verify workflow completes
```

**Scenario 4: Error Handling**
```
1. Start workflow with input
2. Provide invalid input
3. Verify error shown
4. Provide valid input
5. Verify workflow completes
```

**Scenario 5: Resume Multiple Times**
```
1. Start workflow
2. Pause at input A, provide input
3. Pause at input B, provide input
4. Verify both inputs captured correctly
5. Verify final result
```

## Test Data Fixtures

### User Input Request

```rust
fn create_test_input_request() -> UserInputRequest {
    UserInputRequest {
        id: "test-request-1".to_string(),
        task_execution_id: "test-task-1".to_string(),
        workflow_execution_id: "test-workflow-1".to_string(),
        prompt_text: "Please enter your name".to_string(),
        input_type: InputType::String,
        required: true,
        default_value: None,
        validation_rules: serde_json::json!({}),
        status: InputRequestStatus::Pending,
        created_at: chrono::Utc::now(),
        fulfilled_at: None,
        fulfilled_value: None,
    }
}
```

### Task with Input

```rust
fn create_test_task_with_input() -> TaskExecution {
    TaskExecution {
        id: "test-task-1".to_string(),
        workflow_id: "test-workflow-1".to_string(),
        name: "Get User Input".to_string(),
        status: TaskStatus::WaitingForInput,
        output: None,
        error: None,
        created_at: chrono::Utc::now(),
        completed_at: None,
        user_input: None,
        input_request_id: Some("test-request-1".to_string()),
        prompt_id: None,
    }
}
```

## Test Coverage Targets

### Unit Tests
- **Coverage**: >90% for new code
- **Critical paths**: 100%
- **Error paths**: 100%

### Integration Tests
- **All scenarios covered**: Yes
- **Happy path**: 100%
- **Error conditions**: 100%

### Performance Tests
- **Input request creation**: <10ms
- **Input validation**: <1ms
- **Resume operation**: <100ms

## Test Execution

### Running All Tests

```bash
# Persistence
cd persistence && cargo test

# Engine
cd engine && cargo test

# Core
cd core && cargo test

# CLI
cd cli && cargo test

# GUI
cd gui && cargo test
```

### Running Specific Test Files

```bash
# User input specific
cargo test --test user_input_tests

# Integration only
cargo test --test integration_tests

# Logging only
cargo test --test logging_tests
```

## Continuous Integration

### Pre-Commit Checks
1. All unit tests pass
2. No linter warnings
3. No compiler warnings
4. Code formatted (rustfmt)

### CI Pipeline
1. Run all unit tests
2. Run integration tests
3. Run concurrency tests
4. Run performance tests
5. Generate coverage report
6. Fail if coverage drops below threshold

## Debugging Tests

### Test Failures

**Common Issues**:
1. Database not cleaned up → Add teardown
2. Race conditions → Use deterministic timing
3. Missing mocks → Add mock implementations

**Debug Commands**:
```bash
# Verbose output
cargo test -- --nocapture --test-threads=1

# Specific test
cargo test test_specific_function

# Show output
cargo test --test test_file -- --exact --nocapture
```

## Test Maintenance

### Adding New Tests
1. Follow existing patterns
2. Use fixtures for data setup
3. Clean up after each test
4. Add appropriate logging
5. Document complex test logic

### Reviewing Test Coverage
```bash
cargo test --all-features --coverage
```

### Updating Tests
- Update when API changes
- Update when behavior changes
- Keep in sync with implementation
- Remove obsolete tests

