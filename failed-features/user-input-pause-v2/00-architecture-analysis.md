# Architecture Analysis - Current Codebase State

## Overview

This document analyzes the current codebase architecture to understand the foundation for implementing the user input pause feature. The analysis focuses on identifying code duplication, understanding the execution flow, and mapping out the components that will need modification.

## Current Architecture

### Core Components

#### 1. Type System (`core/src/types.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}
```

**Current State**: Clean, only 4 status variants
**Future Change**: Add `WaitingForInput` variant

#### 2. Execution Context (`core/src/execution/context.rs`)
```rust
pub struct ExecutionContext {
    current_task_id: Option<String>,
    per_task_logs: HashMap<String, Vec<String>>,
    output_logs: Vec<String>,
    tasks: Vec<TaskInfo>,
    output_callback: Option<OutputCallback>,
    audit_store: Option<Arc<dyn crate::AuditStore>>,
    execution_id: String,
    workflow_name: String,
    task_start_times: HashMap<String, String>,
}
```

**Current State**: Clean, no user input methods
**Future Change**: Add `pause_for_input()` and `resume_task()` methods

#### 3. Task Handlers (`core/src/engine/handlers/`)

**CRITICAL ISSUE IDENTIFIED**: Massive code duplication

Both `CliCommandHandler` and `CursorAgentHandler` contain identical code blocks:

##### Task Start Persistence (Lines 113-137 in cli_command.rs, 179-203 in cursor_agent.rs)
```rust
if let Ok(ctx) = self.context.lock() {
    if let Some(store) = ctx.get_store() {
        let task_exec = crate::persistence::models::TaskExecution {
            execution_id: ctx.get_execution_id(),
            task_id: task_id.to_string(),
            task_name: task_id.to_string(),
            status: TaskStatus::InProgress,
            logs: ctx.get_task_logs(task_id),
            start_timestamp: ctx.get_task_start_time(task_id),
            end_timestamp: String::new(),
        };
        drop(ctx);

        let span = tracing::debug_span!("save_task_start_bg", task_id = %task_id);
        tokio::spawn(
            async move {
                trace!("Saving task start state");
                if let Err(e) = store.save_task_execution(&task_exec).await {
                    error!(error = %e, "Failed to save task start");
                }
            }
            .instrument(span),
        );
    }
}
```

##### Task Failure Persistence (Lines 176-200 in cli_command.rs, 237-261 in cursor_agent.rs)
```rust
if let Ok(ctx) = self.context.lock() {
    if let Some(store) = ctx.get_store() {
        let task_exec = crate::persistence::models::TaskExecution {
            execution_id: ctx.get_execution_id(),
            task_id: task_id.to_string(),
            task_name: task_id.to_string(),
            status: TaskStatus::Failed,
            logs: ctx.get_task_logs(task_id),
            start_timestamp: ctx.get_task_start_time(task_id),
            end_timestamp: chrono::Utc::now().to_rfc3339(),
        };
        drop(ctx);

        let span = tracing::debug_span!("save_task_failed_bg", task_id = %task_id);
        tokio::spawn(
            async move {
                trace!("Saving failed task state");
                if let Err(e) = store.save_task_execution(&task_exec).await {
                    error!(error = %e, "Failed to save failed task");
                }
            }
            .instrument(span),
        );
    }
}
```

##### Task Completion Persistence (Lines 241-265 in cli_command.rs, 303-327 in cursor_agent.rs)
```rust
if let Ok(ctx) = self.context.lock() {
    if let Some(store) = ctx.get_store() {
        let task_exec = crate::persistence::models::TaskExecution {
            execution_id: ctx.get_execution_id(),
            task_id: task_id.to_string(),
            task_name: task_id.to_string(),
            status: TaskStatus::Complete,
            logs: ctx.get_task_logs(task_id),
            start_timestamp: ctx.get_task_start_time(task_id),
            end_timestamp: chrono::Utc::now().to_rfc3339(),
        };
        drop(ctx);

        let span = tracing::debug_span!("save_task_complete_bg", task_id = %task_id);
        tokio::spawn(
            async move {
                trace!("Saving task completion state");
                if let Err(e) = store.save_task_execution(&task_exec).await {
                    error!(error = %e, "Failed to save task completion");
                }
            }
            .instrument(span),
        );
    }
}
```

**Total Duplication**: ~150 lines of identical code across 2 handlers

### Execution Flow

#### 1. Workflow Execution (`core/src/engine/execute.rs`)
```rust
pub async fn execute_workflow_from_content(
    workflow_data: &str,
    output_callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError>
```

**Current Flow**:
1. Parse workflow JSON
2. Create ExecutionContext
3. Register custom handlers (CliCommandHandler, CursorAgentHandler)
4. Execute via dataflow engine
5. Process results and save to database

**Future Change**: Add resume functionality

#### 2. Task Execution
Each task goes through:
1. **Start**: Handler calls `logger.start_task(task_id)`
2. **Execute**: Handler performs actual work
3. **End**: Handler calls `logger.end_task(task_id)`
4. **Persistence**: Handler saves task state to database

**Current Issue**: Persistence code is duplicated in every handler

### Database Schema

#### Workflow Metadata (`core/src/persistence/models.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub workflow_name: String,
    pub start_timestamp: String,
    pub end_timestamp: Option<String>,
    pub status: WorkflowStatus,
    pub task_ids: Vec<String>,
}
```

**Current State**: Clean, no pause-related fields
**Future Change**: Add `is_paused` and `paused_task_id` fields

#### Task Execution
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub execution_id: String,
    pub task_id: String,
    pub task_name: String,
    pub status: TaskStatus,
    pub logs: Vec<String>,
    pub start_timestamp: String,
    pub end_timestamp: String,
}
```

**Current State**: Clean, supports all current TaskStatus variants
**Future Change**: Support `WaitingForInput` status

### GUI Architecture

#### Execution Details Page (`gui/src/pages/executions/details/page.rs`)
**Current State**: Clean, no user input handling
**Future Change**: Add user input panel and resume button

#### Task Details Panel (`gui/src/pages/executions/details/components/task_details_panel.rs`)
**Current State**: Clean, displays task status
**Future Change**: Show "Waiting for Input" status

## Code Duplication Analysis

### Duplication Statistics
- **Total Duplicated Lines**: ~150 lines
- **Files Affected**: 2 (cli_command.rs, cursor_agent.rs)
- **Duplication Pattern**: 3 identical blocks per file
- **Block Sizes**: ~25 lines each

### Duplication Impact
1. **Maintenance Burden**: Changes must be made in 6 places
2. **Bug Risk**: Inconsistencies between handlers
3. **Testing Complexity**: Must test 6 separate code paths
4. **Future Features**: Adding user input pause would triple duplication

### Refactoring Opportunity
Create `TaskPersistenceHelper` to eliminate all duplication:
- **Reduction**: ~150 lines → ~30 lines total
- **Single Point of Truth**: One implementation for all handlers
- **Easier Testing**: Test persistence logic once
- **Future-Proof**: New features added in one place

## Error Handling Patterns

### Current Error Types
- `CoreError`: Used throughout core library
- `DataflowError`: Used by dataflow engine
- `DataflowError::function_execution()`: Used to wrap CoreError

### Error Propagation
```rust
// In handlers
let result = TaskExecutor::execute(self, input, &logger)
    .await
    .map_err(|e| DataflowError::function_execution(e.to_string(), None))?;
```

**Pattern**: Handlers convert CoreError to DataflowError for dataflow engine

## GUI State Management

### Current State Management
- Uses Dioxus reactive framework
- State managed through `use_signal` hooks
- No complex state interactions currently

### Potential Issues
- **use_effect Dependencies**: Must be carefully managed to avoid infinite loops
- **State Updates**: Should not trigger database queries in effects
- **Error Boundaries**: Need proper error handling for async operations

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `serde`: Serialization
- `tracing`: Logging
- `dataflow-rs`: Workflow engine
- `redb`: Database

### GUI Dependencies
- `dioxus`: Reactive framework
- `tailwindcss`: Styling

## Architecture Recommendations

### 1. Mandatory Refactoring (Phase 0)
- **Eliminate code duplication** before any feature work
- **Create TaskPersistenceHelper** for shared persistence logic
- **Test refactoring thoroughly** to ensure no behavior changes

### 2. Incremental Development
- **Phase-by-phase implementation** with testing at each step
- **No use_effect hooks** until basic functionality works
- **Extensive logging** for debugging

### 3. Error Handling
- **Consistent error types** throughout (CoreError)
- **Proper error propagation** from handlers to GUI
- **Comprehensive error recovery** mechanisms

### 4. State Management
- **Simple GUI state** with minimal complexity
- **Separate business logic** from GUI state
- **Proper effect dependencies** to avoid infinite loops

## Conclusion

The current codebase is clean and well-structured, but has significant code duplication in task handlers. The user input pause feature can be implemented successfully by:

1. **First**: Eliminating code duplication (Phase 0)
2. **Then**: Adding features incrementally with proper testing
3. **Finally**: Ensuring clean architecture for future maintenance

The key is to follow the phased approach exactly and test each phase independently before proceeding to the next.
