# Phase 1: Sequential Execution Engine

## Overview

Build a basic workflow execution engine that can run tasks sequentially, replacing the core functionality of `dataflow-rs`.

## Goals

- Replace `dataflow-rs::Engine` with custom execution loop
- Execute tasks in dependency order
- Maintain existing task handler interface
- Preserve all current functionality

## Current State Analysis

### What We Have ✅
- Task handlers (`CliCommandHandler`, `CursorAgentHandler`)
- Execution context with logging and state management
- Database persistence layer
- Task status tracking (`Pending`, `InProgress`, `Complete`, `Failed`)
- Error handling and logging

### What We Need to Build 🔧
- Custom execution loop
- Task dependency resolution
- Workflow state management
- Integration with existing handlers

## Implementation Plan

### 1. Core Engine Structure

```rust
pub struct CustomWorkflowEngine {
    task_handlers: HashMap<String, Box<dyn TaskHandler>>,
    store: Arc<dyn AuditStore>,
}

pub trait TaskHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, CoreError>;
}
```

### 2. Task Definition

```rust
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub function: TaskFunction,
    pub status: TaskStatus,
}

#[derive(Debug, Clone)]
pub enum TaskFunction {
    CliCommand { command: String, args: Vec<String> },
    CursorAgent { prompt: String, config: Value },
    Custom { name: String, input: Value },
}
```

### 3. Execution Loop

```rust
impl CustomWorkflowEngine {
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, CoreError> {
        let execution_id = Uuid::new_v4().to_string();
        let context = self.create_execution_context(&workflow, &execution_id).await?;
        
        // Execute tasks in dependency order
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks = workflow.tasks.clone();
        
        while !remaining_tasks.is_empty() {
            let ready_tasks = self.get_ready_tasks(&remaining_tasks, &completed_tasks);
            
            if ready_tasks.is_empty() {
                return Err(CoreError::WorkflowExecution("Circular dependency detected".to_string()));
            }
            
            for task in ready_tasks {
                self.execute_task(&context, &task).await?;
                completed_tasks.insert(task.id.clone());
                remaining_tasks.retain(|t| t.id != task.id);
            }
        }
        
        self.build_workflow_result(&context).await
    }
}
```

### 4. Task Execution

```rust
impl CustomWorkflowEngine {
    async fn execute_task(&self, context: &Arc<Mutex<ExecutionContext>>, task: &Task) -> Result<(), CoreError> {
        let handler = self.get_handler(&task.function)?;
        
        // Start task
        context.lock().unwrap().start_task(&task.id);
        
        // Execute task
        let result = handler.execute(context, task).await?;
        
        // End task
        context.lock().unwrap().end_task(&task.id);
        
        // Update status
        context.lock().unwrap().update_task_status(&task.id, TaskStatus::Complete);
        
        Ok(())
    }
}
```

## Migration Strategy

### Step 1: Create New Engine Module
- Create `core/src/engine/custom_engine.rs`
- Implement basic `CustomWorkflowEngine` struct
- Add trait definitions for `TaskHandler`

### Step 2: Adapt Existing Handlers
- Make `CliCommandHandler` and `CursorAgentHandler` implement `TaskHandler`
- Preserve existing functionality
- Add error handling

### Step 3: Update Execution Entry Point
- Modify `execute_workflow_from_content()` to use custom engine
- Keep dataflow-rs as fallback initially
- Add feature flag for engine selection

### Step 4: Testing
- Create comprehensive tests
- Compare results with dataflow-rs
- Ensure all existing workflows work

## File Changes

### New Files
- `core/src/engine/custom_engine.rs` - Main engine implementation
- `core/src/engine/task_handler.rs` - Task handler trait
- `core/tests/custom_engine_tests.rs` - Engine tests

### Modified Files
- `core/src/engine/execute.rs` - Add custom engine option
- `core/src/engine/handlers/cli_command.rs` - Implement TaskHandler
- `core/src/engine/handlers/cursor_agent.rs` - Implement TaskHandler

## Success Criteria

- [ ] All existing workflows execute correctly
- [ ] Task dependencies are respected
- [ ] Error handling works as expected
- [ ] Performance is comparable to dataflow-rs
- [ ] All tests pass
- [ ] No regression in functionality

## Testing Strategy

### Unit Tests
- Test task dependency resolution
- Test error handling
- Test task execution

### Integration Tests
- Test complete workflow execution
- Compare with dataflow-rs results
- Test error scenarios

### Performance Tests
- Benchmark execution time
- Memory usage comparison
- Concurrent workflow execution

## Risks and Mitigation

### Risk: Breaking Existing Functionality
**Mitigation**: Keep dataflow-rs as fallback, extensive testing

### Risk: Performance Regression
**Mitigation**: Benchmark early and often, optimize critical paths

### Risk: Complex Dependency Resolution
**Mitigation**: Start with simple cases, add complexity gradually

## Timeline

- **Week 1**: Core engine structure and basic execution
- **Week 2**: Handler adaptation and testing
- **Week 3**: Integration and performance optimization

## Next Phase

Once Phase 1 is complete, we'll add user input handling in Phase 2, building on the solid foundation of the sequential execution engine.
