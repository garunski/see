# Core Layer Specification - User Input

## Overview

This document specifies the core layer changes for user input support, including bridge conversions, API extensions, and state management.

## Bridge Layer

### User Input Bridge Conversions

**File**: `core/src/bridge/user_input.rs`

```rust
use engine::UserInputRequest as EngineUserInputRequest;
use persistence::UserInputRequest as PersistenceUserInputRequest;

/// Convert persistence UserInputRequest to engine type
pub fn persistence_to_engine_input_request(
    input: &PersistenceUserInputRequest,
) -> EngineUserInputRequest {
    EngineUserInputRequest {
        id: input.id.clone(),
        task_execution_id: input.task_execution_id.clone(),
        workflow_execution_id: input.workflow_execution_id.clone(),
        prompt_text: input.prompt_text.clone(),
        input_type: match input.input_type {
            persistence::InputType::String => engine::InputType::String,
            persistence::InputType::Number => engine::InputType::Number,
            persistence::InputType::Boolean => engine::InputType::Boolean,
        },
        required: input.required,
        default_value: input.default_value.clone(),
        validation_rules: input.validation_rules.clone(),
        status: match input.status {
            persistence::InputRequestStatus::Pending => engine::InputRequestStatus::Pending,
            persistence::InputRequestStatus::Fulfilled => engine::InputRequestStatus::Fulfilled,
        },
        created_at: input.created_at,
        fulfilled_at: input.fulfilled_at,
        fulfilled_value: input.fulfilled_value.clone(),
    }
}

/// Convert engine UserInputRequest to persistence type
pub fn engine_to_persistence_input_request(
    input: &EngineUserInputRequest,
) -> PersistenceUserInputRequest {
    PersistenceUserInputRequest {
        id: input.id.clone(),
        task_execution_id: input.task_execution_id.clone(),
        workflow_execution_id: input.workflow_execution_id.clone(),
        prompt_text: input.prompt_text.clone(),
        input_type: match input.input_type {
            engine::InputType::String => persistence::InputType::String,
            engine::InputType::Number => persistence::InputType::Number,
            engine::InputType::Boolean => persistence::InputType::Boolean,
        },
        required: input.required,
        default_value: input.default_value.clone(),
        validation_rules: input.validation_rules.clone(),
        status: match input.status {
            engine::InputRequestStatus::Pending => persistence::InputRequestStatus::Pending,
            engine::InputRequestStatus::Fulfilled => persistence::InputRequestStatus::Fulfilled,
        },
        created_at: input.created_at,
        fulfilled_at: input.fulfilled_at,
        fulfilled_value: input.fulfilled_value.clone(),
    }
}
```

### Bridge Registration

**File**: `core/src/bridge/mod.rs`

```rust
pub mod audit;
pub mod execution;
pub mod task;
pub mod user_input; // NEW
pub mod workflow;
```

## API Layer

### Input Management API

**File**: `core/src/api/input.rs`

```rust
use crate::errors::CoreError;
use crate::store_singleton::get_global_store;
use persistence::{TaskExecution, TaskStatus, UserInputRequest};
use tracing::{debug, error, info};

/// Provide user input for a waiting task
pub async fn provide_user_input(
    execution_id: &str,
    task_id: &str,
    input_value: String,
) -> Result<(), CoreError> {
    info!(
        execution_id = %execution_id,
        task_id = %task_id,
        "Providing user input for task"
    );

    let store = get_global_store()?;

    // Step 1: Get the task execution
    let execution = store
        .get_workflow_execution(execution_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;

    let task = execution
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| CoreError::TaskNotFound(task_id.to_string()))?;

    // Step 2: Validate task status
    if task.status != TaskStatus::WaitingForInput {
        return Err(CoreError::Execution(format!(
            "Task {} is not waiting for input (status: {:?})",
            task_id, task.status
        )));
    }

    // Step 3: Get input request
    let input_request = store
        .get_input_request_by_task(task_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::Execution("Input request not found".to_string()))?;

    // Step 4: Validate input type
    validate_input_value(&input_value, &input_request.input_type)?;

    // Step 5: Update task with input
    let mut updated_task = task.clone();
    updated_task.user_input = Some(input_value.clone());
    updated_task.status = TaskStatus::InProgress;

    store
        .save_task_execution(updated_task.clone())
        .await
        .map_err(CoreError::Persistence)?;

    // Step 6: Mark input request as fulfilled
    store
        .fulfill_input_request(&input_request.id, input_value.clone())
        .await
        .map_err(CoreError::Persistence)?;

    // Step 7: Resume task execution via engine
    let engine = engine::WorkflowEngine::new();
    let _result = engine
        .resume_task_with_input(execution_id, task_id, input_value)
        .await
        .map_err(CoreError::Engine)?;

    // Step 8: Continue workflow execution
    // TODO: Implement workflow continuation logic

    info!("Task {} resumed successfully with input", task_id);
    Ok(())
}

/// Get pending input requests for a workflow
pub async fn get_pending_inputs(workflow_id: &str) -> Result<Vec<UserInputRequest>, CoreError> {
    let store = get_global_store()?;

    store
        .get_pending_inputs_for_workflow(workflow_id)
        .await
        .map_err(CoreError::Persistence)
}

/// Get all tasks waiting for input in a workflow
pub async fn get_tasks_waiting_for_input(
    workflow_id: &str,
) -> Result<Vec<TaskExecution>, CoreError> {
    let store = get_global_store()?;

    store
        .get_tasks_waiting_for_input_in_workflow(workflow_id)
        .await
        .map_err(CoreError::Persistence)
}

/// Validate input value against input type
fn validate_input_value(value: &str, input_type: &persistence::InputType) -> Result<(), CoreError> {
    match input_type {
        persistence::InputType::String => Ok(()),  // Any string is valid
        persistence::InputType::Number => {
            value.parse::<f64>()
                .map_err(|_| CoreError::Execution("Invalid number format".to_string()))
                .map(|_| ())
        }
        persistence::InputType::Boolean => {
            match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" | "yes" | "no" => Ok(()),
                _ => Err(CoreError::Execution("Invalid boolean format".to_string())),
            }
        }
    }
}
```

### API Registration

**File**: `core/src/api/mod.rs`

```rust
pub mod defaults;
pub mod execution;
pub mod init;
pub mod input; // NEW
pub mod resume;
```

## Execution API Modifications

### Enhanced Execution API

**File**: `core/src/api/execution.rs`

**Modified**: `execute_workflow_by_id()` method

Add handling for paused workflows:

```rust
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    // ... existing steps ...

    // Step 6: Execute Workflow Through Engine
    let engine = WorkflowEngine::new();
    let engine_result = engine
        .execute_workflow(engine_workflow)
        .await
        .map_err(CoreError::Engine)?;

    // NEW: Check if workflow is waiting for input
    let has_input_waiting = engine_result.tasks.iter()
        .any(|t| t.status == engine::TaskStatus::WaitingForInput);

    if has_input_waiting {
        info!("Workflow paused - waiting for user input");
        
        // Update execution status to indicate waiting
        let mut updated_execution = initial_execution.clone();
        updated_execution.status = persistence::WorkflowStatus::Running;
        
        store
            .save_workflow_execution(updated_execution)
            .await
            .map_err(CoreError::Persistence)?;

        // Return special status indicating waiting for input
        return Ok(WorkflowResult {
            success: false,
            workflow_name: engine_result.workflow_name,
            execution_id,
            tasks: engine_result.tasks,
            audit_trail: engine_result.audit_trail,
            per_task_logs: engine_result.per_task_logs,
            errors: vec!["Waiting for user input".to_string()],
        });
    }

    // ... existing completion logic ...
}
```

## Resume API Enhancements

### Enhanced Resume Logic

**File**: `core/src/api/resume.rs`

**Modified**: `resume_task()` method

```rust
pub async fn resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError> {
    info!("Resuming task {} in execution {}", task_id, execution_id);

    let store = get_global_store()?;
    let execution = store
        .get_workflow_execution(execution_id)
        .await
        .map_err(CoreError::Persistence)?
        .ok_or_else(|| CoreError::WorkflowNotFound(execution_id.to_string()))?;

    let task = execution
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| CoreError::TaskNotFound(task_id.to_string()))?;

    // NEW: Check if task has input value
    if task.status == TaskStatus::WaitingForInput {
        // If has input, continue; otherwise error
        if task.user_input.is_none() {
            return Err(CoreError::Execution(
                "Task is waiting for input but no input provided".to_string(),
            ));
        }
    }

    // Validate Task Status
    if task.status != TaskStatus::WaitingForInput {
        return Err(CoreError::Execution(format!(
            "Task {} is not waiting for input (status: {:?})",
            task_id, task.status
        )));
    }

    // ... existing resume logic ...

    Ok(())
}
```

## Error Handling

### Extended Error Types

**File**: `core/src/errors.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    // ... existing errors ...

    #[error("Invalid input type: {0}")]
    InvalidInputType(String),

    #[error("Input value required but not provided")]
    InputRequired,

    #[error("Input validation failed: {0}")]
    InputValidationFailed(String),

    #[error("Task is not waiting for input")]
    TaskNotWaitingForInput,

    #[error("Workflow is waiting for user input")]
    WorkflowWaitingForInput,
}
```

## Logging Requirements

### API Level Logging

```rust
// provide_user_input
info!("Providing user input for task");
debug!("Input validation passed");
trace!("Input value: {}", value);
error!("Input validation failed");

// get_pending_inputs
debug!("Fetching pending inputs for workflow");
info!("Found {} pending inputs", count);

// execute_workflow_by_id
info!("Workflow paused - waiting for user input");
debug!("Workflow has {} tasks waiting for input", count);
```

## Testing Requirements

### Test Files

1. **`core/tests/api/input_tests.rs`**
   - provide_user_input happy path
   - Input validation tests
   - Error cases
   - get_pending_inputs tests

2. **`core/tests/bridge/user_input_tests.rs`**
   - Bridge conversions
   - Type mapping
   - Serialization round-trips

### Test Coverage

- Input validation for all types
- Error handling for all cases
- State transitions
- Integration with persistence and engine

