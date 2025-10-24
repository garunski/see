// User Input Custom Workflow Engine Implementation
// Phase 2: Pause/Resume Functionality

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};

// Enhanced types for pause/resume
#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,
}

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

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub function: TaskFunction,
    pub status: TaskStatus,
    pub pause_config: Option<PauseConfig>,
}

#[derive(Debug, Clone)]
pub enum TaskFunction {
    CliCommand { command: String, args: Vec<String> },
    CursorAgent { prompt: String, config: Value },
    Custom { name: String, input: Value },
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow: Workflow,
    pub state: WorkflowState,
    pub context: Arc<Mutex<ExecutionContext>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub execution_id: String,
    pub tasks: Vec<TaskInfo>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
    pub state: WorkflowState,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}

// Enhanced execution context
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_name: String,
    pub current_task_id: Option<String>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub output_logs: Vec<String>,
    pub tasks: Vec<TaskInfo>,
    pub user_inputs: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(workflow: &Workflow, execution_id: String) -> Self {
        let tasks: Vec<TaskInfo> = workflow.tasks.iter()
            .map(|task| TaskInfo {
                id: task.id.clone(),
                name: task.name.clone(),
                status: TaskStatus::Pending,
            })
            .collect();

        Self {
            execution_id,
            workflow_name: workflow.name.clone(),
            current_task_id: None,
            per_task_logs: HashMap::new(),
            output_logs: Vec::new(),
            tasks,
            user_inputs: HashMap::new(),
        }
    }

    pub fn log(&mut self, message: &str) {
        let log_entry = format!("[{}] {}", 
            Utc::now().format("%H:%M:%S%.3f"), 
            message
        );
        
        self.output_logs.push(log_entry.clone());
        
        if let Some(ref task_id) = self.current_task_id {
            self.per_task_logs
                .entry(task_id.clone())
                .or_insert_with(Vec::new)
                .push(log_entry);
        }
    }

    pub fn start_task(&mut self, task_id: &str) {
        self.current_task_id = Some(task_id.to_string());
        self.log(&format!("Starting task: {}", task_id));
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::InProgress;
        }
    }

    pub fn end_task(&mut self, task_id: &str) {
        self.log(&format!("Completed task: {}", task_id));
        self.current_task_id = None;
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Complete;
        }
    }

    pub fn pause_for_input(&mut self, task_id: &str, prompt: &str) -> Result<(), String> {
        self.log(&format!("⏸️  Task {} paused for user input: {}", task_id, prompt));
        self.current_task_id = None;
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::WaitingForInput;
        }
        
        Ok(())
    }

    pub fn resume_task(&mut self, task_id: &str) -> Result<(), String> {
        self.log(&format!("▶️  Task {} resumed from user input pause", task_id));
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if task.status != TaskStatus::WaitingForInput {
                return Err(format!("Task {} is not waiting for input", task_id));
            }
            task.status = TaskStatus::InProgress;
        }
        
        self.current_task_id = Some(task_id.to_string());
        Ok(())
    }

    pub fn submit_user_input(&mut self, variable: &str, value: &str) {
        self.user_inputs.insert(variable.to_string(), value.to_string());
        self.log(&format!("User input received: {} = {}", variable, value));
    }

    pub fn get_user_input(&self, variable: &str) -> Option<&String> {
        self.user_inputs.get(variable)
    }

    pub fn has_waiting_tasks(&self) -> bool {
        self.tasks.iter().any(|t| t.status == TaskStatus::WaitingForInput)
    }

    pub fn get_waiting_tasks(&self) -> Vec<&TaskInfo> {
        self.tasks.iter().filter(|t| t.status == TaskStatus::WaitingForInput).collect()
    }
}

// Task handler trait with pause support
pub trait TaskHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, String>;
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    Completed(Value),
    Paused { task_id: String, reason: String },
    Failed(String),
}

// Enhanced CLI Command Handler
pub struct CliCommandHandler;

impl CliCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl TaskHandler for CliCommandHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, String> {
        if let TaskFunction::CliCommand { command, args } = &task.function {
            let mut cmd = tokio::process::Command::new(command);
            cmd.args(args);
            
            let output = cmd.output().await
                .map_err(|e| format!("Failed to execute command: {}", e))?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stdout.is_empty() {
                context.lock().unwrap().log(&format!("Output: {}", stdout.trim()));
            }
            
            if !stderr.is_empty() {
                context.lock().unwrap().log(&format!("Error: {}", stderr.trim()));
            }
            
            if !output.status.success() {
                return Ok(TaskResult::Failed(format!(
                    "Command failed with exit code: {:?}",
                    output.status.code()
                )));
            }
            
            // Check if task should pause for input
            if let Some(pause_config) = &task.pause_config {
                context.lock().unwrap().pause_for_input(&task.id, &pause_config.prompt)?;
                return Ok(TaskResult::Paused {
                    task_id: task.id.clone(),
                    reason: pause_config.prompt.clone(),
                });
            }
            
            Ok(TaskResult::Completed(Value::String(stdout.trim().to_string())))
        } else {
            Err("Invalid task function type".to_string())
        }
    }
}

// Enhanced Cursor Agent Handler
pub struct CursorAgentHandler;

impl CursorAgentHandler {
    pub fn new() -> Self {
        Self
    }
}

impl TaskHandler for CursorAgentHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, String> {
        if let TaskFunction::CursorAgent { prompt, .. } = &task.function {
            context.lock().unwrap().log(&format!("Cursor Agent prompt: {}", prompt));
            
            // Simulate cursor agent execution
            sleep(Duration::from_millis(100)).await;
            
            // Check if task should pause for input
            if let Some(pause_config) = &task.pause_config {
                context.lock().unwrap().pause_for_input(&task.id, &pause_config.prompt)?;
                return Ok(TaskResult::Paused {
                    task_id: task.id.clone(),
                    reason: pause_config.prompt.clone(),
                });
            }
            
            Ok(TaskResult::Completed(Value::String("Cursor agent response".to_string())))
        } else {
            Err("Invalid task function type".to_string())
        }
    }
}

// Workflow execution store (in-memory for this example)
pub struct WorkflowExecutionStore {
    executions: Arc<Mutex<HashMap<String, WorkflowExecution>>>,
}

impl WorkflowExecutionStore {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn save_execution(&self, execution: WorkflowExecution) -> Result<(), String> {
        let mut executions = self.executions.lock().unwrap();
        executions.insert(execution.id.clone(), execution);
        Ok(())
    }
    
    pub async fn get_execution(&self, execution_id: &str) -> Result<Option<WorkflowExecution>, String> {
        let executions = self.executions.lock().unwrap();
        Ok(executions.get(execution_id).cloned())
    }
    
    pub async fn get_paused_executions(&self) -> Result<Vec<WorkflowExecution>, String> {
        let executions = self.executions.lock().unwrap();
        Ok(executions.values()
            .filter(|exec| matches!(exec.state, WorkflowState::Paused { .. }))
            .cloned()
            .collect())
    }
}

// Enhanced workflow engine with pause/resume
pub struct UserInputWorkflowEngine {
    task_handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>>,
    execution_store: Arc<WorkflowExecutionStore>,
}

impl UserInputWorkflowEngine {
    pub fn new() -> Self {
        let mut handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>> = HashMap::new();
        
        handlers.insert("cli_command".to_string(), Box::new(CliCommandHandler::new()));
        handlers.insert("cursor_agent".to_string(), Box::new(CursorAgentHandler::new()));
        
        Self {
            task_handlers: handlers,
            execution_store: Arc::new(WorkflowExecutionStore::new()),
        }
    }
    
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, String> {
        let execution_id = Uuid::new_v4().to_string();
        let context = Arc::new(Mutex::new(ExecutionContext::new(&workflow, execution_id.clone())));
        
        context.lock().unwrap().log("Starting workflow execution");
        
        // Check if resuming from pause
        if let Some(paused_execution) = self.execution_store.get_execution(&execution_id).await? {
            return self.resume_workflow(paused_execution).await;
        }
        
        // Execute workflow
        let result = self.execute_workflow_tasks(&context, &workflow).await?;
        
        // Save execution state
        let workflow_execution = WorkflowExecution {
            id: execution_id.clone(),
            workflow: workflow.clone(),
            state: result.state.clone(),
            context: context.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.execution_store.save_execution(workflow_execution).await?;
        
        Ok(result)
    }
    
    async fn execute_workflow_tasks(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        workflow: &Workflow,
    ) -> Result<WorkflowResult, String> {
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks = workflow.tasks.clone();
        
        while !remaining_tasks.is_empty() {
            let ready_tasks = self.get_ready_tasks(&remaining_tasks, &completed_tasks);
            
            if ready_tasks.is_empty() {
                return Err("No ready tasks found - possible circular dependency".to_string());
            }
            
            for task in ready_tasks {
                context.lock().unwrap().start_task(&task.id);
                
                let result = self.execute_task(context, &task).await?;
                
                match result {
                    TaskResult::Completed(_) => {
                        context.lock().unwrap().end_task(&task.id);
                        completed_tasks.insert(task.id.clone());
                        remaining_tasks.retain(|t| t.id != task.id);
                    }
                    TaskResult::Paused { task_id, reason } => {
                        // Workflow is paused
                        let context_guard = context.lock().unwrap();
                        return Ok(WorkflowResult {
                            success: false, // Not completed yet
                            workflow_name: context_guard.workflow_name.clone(),
                            task_count: context_guard.tasks.len(),
                            execution_id: context_guard.execution_id.clone(),
                            tasks: context_guard.tasks.clone(),
                            errors: vec![],
                            output_logs: context_guard.output_logs.clone(),
                            state: WorkflowState::Paused {
                                paused_task_id: task_id,
                                pause_reason: reason,
                                paused_at: Utc::now(),
                            },
                        });
                    }
                    TaskResult::Failed(error) => {
                        context.lock().unwrap().fail_task(&task.id, &error);
                        return Err(format!("Task {} failed: {}", task.id, error));
                    }
                }
            }
        }
        
        context.lock().unwrap().log("Workflow execution completed");
        
        // Build completed result
        let context_guard = context.lock().unwrap();
        Ok(WorkflowResult {
            success: true,
            workflow_name: context_guard.workflow_name.clone(),
            task_count: context_guard.tasks.len(),
            execution_id: context_guard.execution_id.clone(),
            tasks: context_guard.tasks.clone(),
            errors: vec![],
            output_logs: context_guard.output_logs.clone(),
            state: WorkflowState::Completed,
        })
    }
    
    async fn execute_task(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, String> {
        let handler_name = match &task.function {
            TaskFunction::CliCommand { .. } => "cli_command",
            TaskFunction::CursorAgent { .. } => "cursor_agent",
            TaskFunction::Custom { name, .. } => name,
        };
        
        let handler = self.task_handlers.get(handler_name)
            .ok_or_else(|| format!("No handler found for task type: {}", handler_name))?;
        
        handler.execute(context, task).await
    }
    
    fn get_ready_tasks(
        &self,
        remaining_tasks: &[Task],
        completed_tasks: &HashSet<String>,
    ) -> Vec<Task> {
        remaining_tasks.iter()
            .filter(|task| {
                !completed_tasks.contains(&task.id) &&
                task.dependencies.iter().all(|dep| completed_tasks.contains(dep))
            })
            .cloned()
            .collect()
    }
    
    pub async fn resume_workflow(&self, execution: WorkflowExecution) -> Result<WorkflowResult, String> {
        let context = execution.context;
        
        // Resume from paused task
        let paused_task_id = match &execution.state {
            WorkflowState::Paused { paused_task_id, .. } => paused_task_id.clone(),
            _ => return Err("Workflow is not paused".to_string()),
        };
        
        context.lock().unwrap().resume_task(&paused_task_id)?;
        
        // Continue execution
        self.execute_workflow_tasks(&context, &execution.workflow).await
    }
    
    pub async fn submit_user_input(
        &self,
        execution_id: &str,
        variable: &str,
        value: &str,
    ) -> Result<(), String> {
        if let Some(execution) = self.execution_store.get_execution(execution_id).await? {
            execution.context.lock().unwrap().submit_user_input(variable, value);
        }
        Ok(())
    }
    
    pub async fn get_paused_workflows(&self) -> Result<Vec<WorkflowExecution>, String> {
        self.execution_store.get_paused_executions().await
    }
}

// Example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workflow with pause points
    let workflow = Workflow {
        id: "user_input_workflow".to_string(),
        name: "User Input Workflow".to_string(),
        tasks: vec![
            Task {
                id: "task1".to_string(),
                name: "First Task".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Starting workflow".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
            Task {
                id: "task2".to_string(),
                name: "Pause for Input".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["This task will pause for input".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: Some(PauseConfig {
                    prompt: "Do you want to continue?".to_string(),
                    variable: "continue_choice".to_string(),
                    input_type: InputType::YesNo,
                }),
            },
            Task {
                id: "task3".to_string(),
                name: "Final Task".to_string(),
                dependencies: vec!["task2".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Workflow completed".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
        ],
    };
    
    // Execute workflow
    let engine = UserInputWorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await?;
    
    println!("Workflow execution result:");
    println!("Success: {}", result.success);
    println!("State: {:?}", result.state);
    println!("Execution ID: {}", result.execution_id);
    
    if matches!(result.state, WorkflowState::Paused { .. }) {
        println!("\nWorkflow is paused for user input!");
        
        // Simulate user input
        engine.submit_user_input(&result.execution_id, "continue_choice", "yes").await?;
        
        // Resume workflow
        let paused_execution = engine.get_paused_workflows().await?;
        if let Some(execution) = paused_execution.first() {
            let final_result = engine.resume_workflow(execution.clone()).await?;
            println!("\nWorkflow resumed and completed!");
            println!("Success: {}", final_result.success);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pause_resume_workflow() {
        let workflow = Workflow {
            id: "test".to_string(),
            name: "Test".to_string(),
            tasks: vec![
                Task {
                    id: "task1".to_string(),
                    name: "Task 1".to_string(),
                    dependencies: vec![],
                    function: TaskFunction::CliCommand {
                        command: "echo".to_string(),
                        args: vec!["hello".to_string()],
                    },
                    status: TaskStatus::Pending,
                    pause_config: Some(PauseConfig {
                        prompt: "Continue?".to_string(),
                        variable: "choice".to_string(),
                        input_type: InputType::YesNo,
                    }),
                },
            ],
        };
        
        let engine = UserInputWorkflowEngine::new();
        let result = engine.execute_workflow(workflow).await.unwrap();
        
        assert!(!result.success); // Should be paused
        assert!(matches!(result.state, WorkflowState::Paused { .. }));
    }
}
