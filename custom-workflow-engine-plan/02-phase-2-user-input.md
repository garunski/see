# Phase 2: User Input Handling

## Overview

Add pause/resume functionality to the custom workflow engine, enabling workflows to wait for user input and resume from the exact pause point.

## Goals

- Implement workflow pausing for user input
- Add resume functionality from pause point
- Persist workflow state across app restarts
- Integrate with existing GUI components

## Current State Analysis

### What We Have ✅
- `TaskStatus::WaitingForInput` enum variant
- `ExecutionContext::pause_for_input()` method
- `ExecutionContext::resume_task()` method
- Database persistence for pause state
- GUI components for pause/resume

### What We Need to Build 🔧
- Pause detection in execution loop
- State persistence during pause
- Resume from exact pause point
- Integration with GUI

## Implementation Plan

### 1. Enhanced Task Definition

```rust
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub function: TaskFunction,
    pub status: TaskStatus,
    pub pause_config: Option<PauseConfig>, // New field
}

#[derive(Debug, Clone)]
pub struct PauseConfig {
    pub prompt: String,
    pub variable: String,
    pub input_type: InputType,
}

#[derive(Debug, Clone)]
pub enum InputType {
    YesNo,
    Text,
    Choice(Vec<String>),
}
```

### 2. Workflow State Management

```rust
#[derive(Debug, Clone)]
pub enum WorkflowState {
    Running,
    Paused { 
        paused_task_id: String,
        pause_reason: String,
        paused_at: DateTime<Utc>,
    },
    Completed,
    Failed,
}

pub struct WorkflowExecution {
    pub id: String,
    pub workflow: Workflow,
    pub state: WorkflowState,
    pub context: Arc<Mutex<ExecutionContext>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 3. Enhanced Execution Loop

```rust
impl CustomWorkflowEngine {
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, CoreError> {
        let execution_id = Uuid::new_v4().to_string();
        let context = self.create_execution_context(&workflow, &execution_id).await?;
        
        // Check if resuming from pause
        if let Some(paused_execution) = self.get_paused_execution(&execution_id).await? {
            return self.resume_workflow(paused_execution).await;
        }
        
        // Execute tasks in dependency order
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks = workflow.tasks.clone();
        
        while !remaining_tasks.is_empty() {
            let ready_tasks = self.get_ready_tasks(&remaining_tasks, &completed_tasks);
            
            for task in ready_tasks {
                // Check if task should pause
                if let Some(pause_config) = &task.pause_config {
                    if self.should_pause_for_input(&task, pause_config).await? {
                        return self.pause_workflow(&execution_id, &task, pause_config).await;
                    }
                }
                
                // Execute task
                self.execute_task(&context, &task).await?;
                completed_tasks.insert(task.id.clone());
                remaining_tasks.retain(|t| t.id != task.id);
            }
        }
        
        self.build_workflow_result(&context).await
    }
}
```

### 4. Pause Implementation

```rust
impl CustomWorkflowEngine {
    async fn pause_workflow(
        &self,
        execution_id: &str,
        task: &Task,
        pause_config: &PauseConfig,
    ) -> Result<WorkflowResult, CoreError> {
        // Update task status
        let mut context = self.get_execution_context(execution_id).await?;
        context.lock().unwrap().pause_for_input(&task.id, &pause_config.prompt)?;
        
        // Save workflow state
        let workflow_execution = WorkflowExecution {
            id: execution_id.to_string(),
            workflow: self.get_workflow(execution_id).await?,
            state: WorkflowState::Paused {
                paused_task_id: task.id.clone(),
                pause_reason: pause_config.prompt.clone(),
                paused_at: Utc::now(),
            },
            context: context.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.save_workflow_execution(&workflow_execution).await?;
        
        // Return partial result
        self.build_partial_workflow_result(&context, WorkflowState::Paused {
            paused_task_id: task.id.clone(),
            pause_reason: pause_config.prompt.clone(),
            paused_at: Utc::now(),
        }).await
    }
}
```

### 5. Resume Implementation

```rust
impl CustomWorkflowEngine {
    pub async fn resume_workflow(&self, execution_id: &str) -> Result<WorkflowResult, CoreError> {
        let mut workflow_execution = self.get_paused_execution(execution_id).await?
            .ok_or_else(|| CoreError::Validation("Workflow not paused".to_string()))?;
        
        // Update state to running
        workflow_execution.state = WorkflowState::Running;
        workflow_execution.updated_at = Utc::now();
        
        // Resume from paused task
        let paused_task_id = match &workflow_execution.state {
            WorkflowState::Paused { paused_task_id, .. } => paused_task_id.clone(),
            _ => return Err(CoreError::Validation("Workflow not paused".to_string())),
        };
        
        // Continue execution from paused task
        self.continue_workflow_execution(&mut workflow_execution, &paused_task_id).await
    }
    
    async fn continue_workflow_execution(
        &self,
        workflow_execution: &mut WorkflowExecution,
        from_task_id: &str,
    ) -> Result<WorkflowResult, CoreError> {
        let context = &workflow_execution.context;
        let workflow = &workflow_execution.workflow;
        
        // Find tasks to execute (from paused task onwards)
        let mut remaining_tasks = workflow.tasks.clone();
        let mut completed_tasks = self.get_completed_tasks(context).await?;
        
        // Remove completed tasks
        remaining_tasks.retain(|t| !completed_tasks.contains(&t.id));
        
        // Continue execution
        while !remaining_tasks.is_empty() {
            let ready_tasks = self.get_ready_tasks(&remaining_tasks, &completed_tasks);
            
            for task in ready_tasks {
                // Check if task should pause again
                if let Some(pause_config) = &task.pause_config {
                    if self.should_pause_for_input(&task, pause_config).await? {
                        return self.pause_workflow(&workflow_execution.id, &task, pause_config).await;
                    }
                }
                
                // Execute task
                self.execute_task(context, &task).await?;
                completed_tasks.insert(task.id.clone());
                remaining_tasks.retain(|t| t.id != task.id);
            }
        }
        
        // Mark as completed
        workflow_execution.state = WorkflowState::Completed;
        workflow_execution.updated_at = Utc::now();
        self.save_workflow_execution(workflow_execution).await?;
        
        self.build_workflow_result(context).await
    }
}
```

### 6. Task Handler Integration

```rust
impl TaskHandler for CliCommandHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, CoreError> {
        // ... existing execution logic ...
        
        // Check if task should pause for input
        if let Some(pause_config) = &task.pause_config {
            if self.should_pause_for_input(task, pause_config).await? {
                // Pause the workflow
                context.lock().unwrap().pause_for_input(&task.id, &pause_config.prompt)?;
                return Ok(TaskResult::Paused {
                    task_id: task.id.clone(),
                    reason: pause_config.prompt.clone(),
                });
            }
        }
        
        // ... continue with normal execution ...
        Ok(TaskResult::Completed(result))
    }
}
```

## Database Schema Updates

### Workflow Executions Table
```sql
CREATE TABLE workflow_executions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    state TEXT NOT NULL, -- 'running', 'paused', 'completed', 'failed'
    paused_task_id TEXT,
    pause_reason TEXT,
    paused_at TIMESTAMP,
    context_data BLOB, -- Serialized ExecutionContext
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

### User Input Responses Table
```sql
CREATE TABLE user_input_responses (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    variable_name TEXT NOT NULL,
    response_value TEXT NOT NULL,
    responded_at TIMESTAMP NOT NULL,
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id)
);
```

## GUI Integration

### 1. Pause Detection
```rust
// In GUI state management
pub struct WorkflowState {
    pub executions: HashMap<String, WorkflowExecution>,
    pub paused_executions: Vec<String>,
}

impl WorkflowState {
    pub fn get_paused_executions(&self) -> Vec<&WorkflowExecution> {
        self.executions.values()
            .filter(|exec| matches!(exec.state, WorkflowState::Paused { .. }))
            .collect()
    }
}
```

### 2. Resume Button
```rust
// In GUI component
#[component]
pub fn PausedWorkflowItem(execution: WorkflowExecution) -> Element {
    let resume_workflow = use_callback(move |execution_id: String| {
        spawn(async move {
            if let Err(e) = custom_engine.resume_workflow(&execution_id).await {
                eprintln!("Failed to resume workflow: {}", e);
            }
        });
    });
    
    rsx! {
        div { class: "paused-workflow-item",
            h3 { "Workflow: {execution.workflow.name}" }
            p { "Paused at: {execution.paused_task_id}" }
            p { "Reason: {execution.pause_reason}" }
            button {
                onclick: move |_| resume_workflow(execution.id.clone()),
                "Resume"
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Test pause detection logic
- Test resume from pause point
- Test state persistence
- Test error handling

### Integration Tests
- Test complete pause/resume cycle
- Test multiple paused workflows
- Test resume after app restart
- Test GUI integration

### End-to-End Tests
- Test user input workflow
- Test pause/resume from GUI
- Test concurrent paused workflows

## Success Criteria

- [ ] Workflows pause at correct tasks
- [ ] Workflows resume from exact pause point
- [ ] State persists across app restarts
- [ ] GUI shows paused workflows
- [ ] Resume functionality works from GUI
- [ ] No data loss during pause/resume
- [ ] Performance is acceptable

## Risks and Mitigation

### Risk: State Corruption During Pause
**Mitigation**: Atomic state updates, comprehensive testing

### Risk: Memory Leaks with Paused Workflows
**Mitigation**: Proper cleanup, memory monitoring

### Risk: Complex State Management
**Mitigation**: Clear state machine, extensive documentation

## Timeline

- **Week 1**: Core pause/resume logic
- **Week 2**: State persistence and database integration
- **Week 3**: GUI integration and testing

## Next Phase

Once Phase 2 is complete, we'll add parallel task execution in Phase 3, building on the pause/resume functionality.
