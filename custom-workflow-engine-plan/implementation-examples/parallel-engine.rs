// Parallel Custom Workflow Engine Implementation
// Phase 3: Parallel Task Execution

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};
use futures::future::join_all;

// Enhanced types for parallel execution
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
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub task_count: usize,
    pub execution_id: String,
    pub tasks: Vec<TaskInfo>,
    pub errors: Vec<String>,
    pub output_logs: Vec<String>,
    pub state: WorkflowState,
    pub execution_time: Duration,
    pub parallel_efficiency: f64,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub execution_time: Duration,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

// Parallel execution configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub max_concurrent_tasks: usize,
    pub enable_work_stealing: bool,
    pub task_timeout: Duration,
    pub enable_monitoring: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            enable_work_stealing: true,
            task_timeout: Duration::from_secs(300),
            enable_monitoring: true,
        }
    }
}

// Concurrency controller
pub struct ConcurrencyController {
    max_concurrent_tasks: usize,
    semaphore: Arc<tokio::sync::Semaphore>,
    running_tasks: Arc<Mutex<HashSet<String>>>,
}

impl ConcurrencyController {
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            max_concurrent_tasks,
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent_tasks)),
            running_tasks: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    pub async fn acquire_task_slot(&self, task_id: &str) -> Result<tokio::sync::SemaphorePermit, String> {
        let permit = self.semaphore.acquire().await
            .map_err(|e| format!("Failed to acquire task slot: {}", e))?;
        
        self.running_tasks.lock().unwrap().insert(task_id.to_string());
        Ok(permit)
    }
    
    pub fn release_task_slot(&self, task_id: &str) {
        self.running_tasks.lock().unwrap().remove(task_id);
    }
    
    pub fn get_running_count(&self) -> usize {
        self.running_tasks.lock().unwrap().len()
    }
    
    pub fn can_start_task(&self) -> bool {
        self.get_running_count() < self.max_concurrent_tasks
    }
}

// Work-stealing executor
pub struct WorkStealingExecutor {
    max_workers: usize,
    concurrency_controller: Arc<ConcurrencyController>,
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    completed_tasks: Arc<Mutex<HashSet<String>>>,
    task_handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>>,
}

impl WorkStealingExecutor {
    pub fn new(
        max_workers: usize,
        concurrency_controller: Arc<ConcurrencyController>,
        task_handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>>,
    ) -> Self {
        Self {
            max_workers,
            concurrency_controller,
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            completed_tasks: Arc::new(Mutex::new(HashSet::new())),
            task_handlers,
        }
    }
    
    pub async fn execute_workflow(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        workflow: &Workflow,
    ) -> Result<(), String> {
        let start_time = Instant::now();
        
        // Initialize task queue with ready tasks
        let ready_tasks = self.get_ready_tasks(&workflow.tasks, &HashSet::new());
        for task in ready_tasks {
            self.task_queue.lock().unwrap().push_back(task);
        }
        
        // Spawn worker threads
        let mut workers = Vec::new();
        for worker_id in 0..self.max_workers {
            let worker = self.spawn_worker(
                worker_id,
                context.clone(),
                workflow.clone(),
            );
            workers.push(worker);
        }
        
        // Wait for all workers to complete
        let results = join_all(workers).await;
        
        // Check for errors
        for result in results {
            if let Err(e) = result {
                return Err(format!("Worker error: {}", e));
            }
        }
        
        let execution_time = start_time.elapsed();
        context.lock().unwrap().log(&format!(
            "Parallel execution completed in {:?}",
            execution_time
        ));
        
        Ok(())
    }
    
    async fn spawn_worker(
        &self,
        worker_id: usize,
        context: Arc<Mutex<ExecutionContext>>,
        workflow: Workflow,
    ) -> Result<(), String> {
        loop {
            // Try to get a task from the queue
            let task = {
                let mut queue = self.task_queue.lock().unwrap();
                queue.pop_front()
            };
            
            if let Some(task) = task {
                // Check if we can start this task
                if self.concurrency_controller.can_start_task() {
                    // Acquire task slot
                    let _permit = self.concurrency_controller.acquire_task_slot(&task.id).await?;
                    
                    // Execute task
                    let result = self.execute_task(&context, &task).await;
                    
                    // Release task slot
                    self.concurrency_controller.release_task_slot(&task.id);
                    
                    // Handle result
                    match result {
                        Ok(TaskResult::Completed(_)) => {
                            self.completed_tasks.lock().unwrap().insert(task.id.clone());
                            
                            // Add newly ready tasks to queue
                            let ready_tasks = self.get_ready_tasks(&workflow.tasks, &self.completed_tasks.lock().unwrap().clone());
                            for ready_task in ready_tasks {
                                self.task_queue.lock().unwrap().push_back(ready_task);
                            }
                        }
                        Ok(TaskResult::Paused { .. }) => {
                            // Handle pause - this would need special handling
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(format!("Task {} failed: {}", task.id, e));
                        }
                    }
                } else {
                    // No available slots, put task back and wait
                    self.task_queue.lock().unwrap().push_front(task);
                    sleep(Duration::from_millis(10)).await;
                }
            } else {
                // No more tasks, check if we're done
                if self.completed_tasks.lock().unwrap().len() == workflow.tasks.len() {
                    break;
                }
                
                // Wait a bit before checking again
                sleep(Duration::from_millis(10)).await;
            }
        }
        
        Ok(())
    }
    
    fn get_ready_tasks(
        &self,
        tasks: &[Task],
        completed: &HashSet<String>,
    ) -> Vec<Task> {
        tasks.iter()
            .filter(|task| {
                !completed.contains(&task.id) &&
                task.dependencies.iter().all(|dep| completed.contains(dep))
            })
            .cloned()
            .collect()
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
}

// Enhanced execution context for parallel execution
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_name: String,
    pub current_task_id: Option<String>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub output_logs: Vec<String>,
    pub tasks: Vec<TaskInfo>,
    pub user_inputs: HashMap<String, String>,
    pub task_times: HashMap<String, (DateTime<Utc>, Option<DateTime<Utc>>)>,
}

impl ExecutionContext {
    pub fn new(workflow: &Workflow, execution_id: String) -> Self {
        let tasks: Vec<TaskInfo> = workflow.tasks.iter()
            .map(|task| TaskInfo {
                id: task.id.clone(),
                name: task.name.clone(),
                status: TaskStatus::Pending,
                execution_time: Duration::from_secs(0),
                start_time: Utc::now(),
                end_time: None,
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
            task_times: HashMap::new(),
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
        
        let start_time = Utc::now();
        self.task_times.insert(task_id.to_string(), (start_time, None));
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::InProgress;
            task.start_time = start_time;
        }
    }

    pub fn end_task(&mut self, task_id: &str) {
        let end_time = Utc::now();
        self.log(&format!("Completed task: {}", task_id));
        
        if let Some((start_time, _)) = self.task_times.get_mut(&task_id.to_string()) {
            *self.task_times.get_mut(&task_id.to_string()).unwrap() = (*start_time, Some(end_time));
        }
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Complete;
            task.end_time = Some(end_time);
            if let Some((start, _)) = self.task_times.get(&task_id) {
                task.execution_time = end_time.signed_duration_since(*start).to_std().unwrap_or_default();
            }
        }
        
        self.current_task_id = None;
    }

    pub fn fail_task(&mut self, task_id: &str, error: &str) {
        self.log(&format!("Failed task: {} - {}", task_id, error));
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Failed;
            task.end_time = Some(Utc::now());
        }
        
        self.current_task_id = None;
    }

    pub fn pause_for_input(&mut self, task_id: &str, prompt: &str) -> Result<(), String> {
        self.log(&format!("⏸️  Task {} paused for user input: {}", task_id, prompt));
        self.current_task_id = None;
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::WaitingForInput;
        }
        
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

// Task handler trait
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

// CLI Command Handler
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

// Cursor Agent Handler
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
            
            // Simulate cursor agent execution with variable delay
            let delay = Duration::from_millis(50 + (task.id.len() as u64 * 10));
            sleep(delay).await;
            
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

// Main parallel workflow engine
pub struct ParallelWorkflowEngine {
    task_handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>>,
    concurrency_controller: Arc<ConcurrencyController>,
    config: ParallelConfig,
}

impl ParallelWorkflowEngine {
    pub fn new(config: ParallelConfig) -> Self {
        let mut handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>> = HashMap::new();
        
        handlers.insert("cli_command".to_string(), Box::new(CliCommandHandler::new()));
        handlers.insert("cursor_agent".to_string(), Box::new(CursorAgentHandler::new()));
        
        let concurrency_controller = Arc::new(ConcurrencyController::new(config.max_concurrent_tasks));
        
        Self {
            task_handlers: handlers,
            concurrency_controller,
            config,
        }
    }
    
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, String> {
        let execution_id = Uuid::new_v4().to_string();
        let context = Arc::new(Mutex::new(ExecutionContext::new(&workflow, execution_id.clone())));
        
        let start_time = Instant::now();
        context.lock().unwrap().log("Starting parallel workflow execution");
        
        // Create work-stealing executor
        let executor = WorkStealingExecutor::new(
            self.config.max_concurrent_tasks,
            self.concurrency_controller.clone(),
            self.task_handlers.clone(),
        );
        
        // Execute workflow
        executor.execute_workflow(&context, &workflow).await?;
        
        let execution_time = start_time.elapsed();
        
        // Calculate parallel efficiency
        let context_guard = context.lock().unwrap();
        let total_task_time: Duration = context_guard.tasks.iter()
            .map(|task| task.execution_time)
            .sum();
        
        let parallel_efficiency = if total_task_time.as_secs_f64() > 0.0 {
            total_task_time.as_secs_f64() / execution_time.as_secs_f64()
        } else {
            0.0
        };
        
        context_guard.log(&format!(
            "Parallel execution completed in {:?} (efficiency: {:.2}%)",
            execution_time,
            parallel_efficiency * 100.0
        ));
        
        // Build result
        Ok(WorkflowResult {
            success: true,
            workflow_name: context_guard.workflow_name.clone(),
            task_count: context_guard.tasks.len(),
            execution_id: context_guard.execution_id.clone(),
            tasks: context_guard.tasks.clone(),
            errors: vec![],
            output_logs: context_guard.output_logs.clone(),
            state: WorkflowState::Completed,
            execution_time,
            parallel_efficiency,
        })
    }
}

// Example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workflow with parallel tasks
    let workflow = Workflow {
        id: "parallel_workflow".to_string(),
        name: "Parallel Workflow".to_string(),
        tasks: vec![
            // Initial task
            Task {
                id: "task1".to_string(),
                name: "Initial Task".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Starting parallel workflow".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
            // Parallel tasks (no dependencies between them)
            Task {
                id: "task2".to_string(),
                name: "Parallel Task 1".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CursorAgent {
                    prompt: "Generate a greeting".to_string(),
                    config: Value::Object(serde_json::Map::new()),
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
            Task {
                id: "task3".to_string(),
                name: "Parallel Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CursorAgent {
                    prompt: "Generate a farewell".to_string(),
                    config: Value::Object(serde_json::Map::new()),
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
            Task {
                id: "task4".to_string(),
                name: "Parallel Task 3".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Processing data".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
            // Final task (depends on all parallel tasks)
            Task {
                id: "task5".to_string(),
                name: "Final Task".to_string(),
                dependencies: vec!["task2".to_string(), "task3".to_string(), "task4".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["All parallel tasks completed".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
            },
        ],
    };
    
    // Create engine with parallel configuration
    let config = ParallelConfig {
        max_concurrent_tasks: 4,
        enable_work_stealing: true,
        task_timeout: Duration::from_secs(30),
        enable_monitoring: true,
    };
    
    let engine = ParallelWorkflowEngine::new(config);
    let result = engine.execute_workflow(workflow).await?;
    
    println!("Parallel workflow executed successfully!");
    println!("Execution ID: {}", result.execution_id);
    println!("Task count: {}", result.task_count);
    println!("Execution time: {:?}", result.execution_time);
    println!("Parallel efficiency: {:.2}%", result.parallel_efficiency * 100.0);
    
    println!("\nTask execution times:");
    for task in result.tasks {
        println!("  {}: {:?}", task.name, task.execution_time);
    }
    
    println!("\nExecution logs:");
    for log in result.output_logs {
        println!("{}", log);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_parallel_execution() {
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
                    pause_config: None,
                },
                Task {
                    id: "task2".to_string(),
                    name: "Task 2".to_string(),
                    dependencies: vec!["task1".to_string()],
                    function: TaskFunction::CliCommand {
                        command: "echo".to_string(),
                        args: vec!["world".to_string()],
                    },
                    status: TaskStatus::Pending,
                    pause_config: None,
                },
            ],
        };
        
        let config = ParallelConfig::default();
        let engine = ParallelWorkflowEngine::new(config);
        let result = engine.execute_workflow(workflow).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.task_count, 2);
        assert!(result.parallel_efficiency > 0.0);
    }
}
