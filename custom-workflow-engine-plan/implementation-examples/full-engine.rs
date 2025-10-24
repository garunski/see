// Full Custom Workflow Engine Implementation
// Complete implementation with all features

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use lru::LruCache;

// Core types
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
    pub retry_config: Option<RetryConfig>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub retry_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_retry_delay: Duration,
    pub retry_on_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_retry_delay: Duration::from_secs(60),
            retry_on_errors: vec!["timeout".to_string(), "network".to_string()],
        }
    }
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
    pub metrics: WorkflowMetrics,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub execution_time: Duration,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub retry_count: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub average_execution_time: Duration,
    pub cache_hit_rate: f64,
    pub memory_usage: usize,
    pub cpu_usage: f64,
}

// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub max_concurrent_workflows: usize,
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
    pub workflow_timeout: Duration,
    pub memory_limit: usize,
    pub cache_size: usize,
    pub enable_compression: bool,
    pub enable_caching: bool,
    pub enable_monitoring: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 100,
            max_concurrent_tasks: 50,
            task_timeout: Duration::from_secs(300),
            workflow_timeout: Duration::from_secs(3600),
            memory_limit: 1024 * 1024 * 1024, // 1GB
            cache_size: 1000,
            enable_compression: true,
            enable_caching: true,
            enable_monitoring: true,
        }
    }
}

// Cached result
#[derive(Debug, Clone)]
pub struct CachedResult {
    pub result: TaskResult,
    pub timestamp: DateTime<Utc>,
    pub ttl: Duration,
}

impl CachedResult {
    pub fn is_expired(&self) -> bool {
        Utc::now().signed_duration_since(self.timestamp) > self.ttl
    }
}

// Task result
#[derive(Debug, Clone)]
pub enum TaskResult {
    Completed(Value),
    Paused { task_id: String, reason: String },
    Failed(String),
}

// Performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<WorkflowMetrics>>,
    task_cache: Arc<Mutex<LruCache<String, CachedResult>>>,
    execution_times: Arc<Mutex<Vec<Duration>>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(WorkflowMetrics {
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                average_execution_time: Duration::from_secs(0),
                cache_hit_rate: 0.0,
                memory_usage: 0,
                cpu_usage: 0.0,
            })),
            task_cache: Arc::new(Mutex::new(LruCache::new(1000))),
            execution_times: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn record_execution_time(&self, duration: Duration) {
        let mut times = self.execution_times.lock().unwrap();
        times.push(duration);
        
        // Keep only last 1000 executions
        if times.len() > 1000 {
            times.remove(0);
        }
        
        // Update average
        let mut metrics = self.metrics.lock().unwrap();
        metrics.average_execution_time = if times.is_empty() {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(
                times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / times.len() as f64
            )
        };
    }
    
    pub fn get_cached_result(&self, cache_key: &str) -> Option<CachedResult> {
        let mut cache = self.task_cache.lock().unwrap();
        cache.get(cache_key).cloned()
    }
    
    pub fn cache_result(&self, cache_key: String, result: TaskResult) {
        let cached = CachedResult {
            result,
            timestamp: Utc::now(),
            ttl: Duration::from_secs(3600), // 1 hour TTL
        };
        
        let mut cache = self.task_cache.lock().unwrap();
        cache.put(cache_key, cached);
    }
    
    pub fn get_metrics(&self) -> WorkflowMetrics {
        self.metrics.lock().unwrap().clone()
    }
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
    pub task_times: HashMap<String, (DateTime<Utc>, Option<DateTime<Utc>>)>,
    pub performance_monitor: Arc<PerformanceMonitor>,
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
                retry_count: 0,
                error_message: None,
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
            performance_monitor: Arc::new(PerformanceMonitor::new()),
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
            task.error_message = Some(error.to_string());
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

// CLI Command Handler with retry and timeout
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
            // Check cache first
            let cache_key = format!("cli_command:{}:{}", command, args.join(" "));
            if let Some(cached) = context.lock().unwrap().performance_monitor.get_cached_result(&cache_key) {
                if !cached.is_expired() {
                    context.lock().unwrap().log(&format!("Using cached result for task: {}", task.id));
                    return Ok(cached.result);
                }
            }
            
            let mut cmd = tokio::process::Command::new(command);
            cmd.args(args);
            
            // Apply timeout if configured
            let timeout_duration = task.timeout.unwrap_or(Duration::from_secs(30));
            let output = timeout(timeout_duration, cmd.output()).await
                .map_err(|_| format!("Command timed out after {:?}", timeout_duration))?
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
            
            let result = TaskResult::Completed(Value::String(stdout.trim().to_string()));
            
            // Cache result
            context.lock().unwrap().performance_monitor.cache_result(cache_key, result.clone());
            
            // Check if task should pause for input
            if let Some(pause_config) = &task.pause_config {
                context.lock().unwrap().pause_for_input(&task.id, &pause_config.prompt)?;
                return Ok(TaskResult::Paused {
                    task_id: task.id.clone(),
                    reason: pause_config.prompt.clone(),
                });
            }
            
            Ok(result)
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

// Main full-featured workflow engine
pub struct FullWorkflowEngine {
    task_handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>>,
    performance_config: PerformanceConfig,
    performance_monitor: Arc<PerformanceMonitor>,
}

impl FullWorkflowEngine {
    pub fn new(config: PerformanceConfig) -> Self {
        let mut handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>> = HashMap::new();
        
        handlers.insert("cli_command".to_string(), Box::new(CliCommandHandler::new()));
        handlers.insert("cursor_agent".to_string(), Box::new(CursorAgentHandler::new()));
        
        Self {
            task_handlers: handlers,
            performance_config: config,
            performance_monitor: Arc::new(PerformanceMonitor::new()),
        }
    }
    
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, String> {
        let execution_id = Uuid::new_v4().to_string();
        let context = Arc::new(Mutex::new(ExecutionContext::new(&workflow, execution_id.clone())));
        
        let start_time = Instant::now();
        context.lock().unwrap().log("Starting full-featured workflow execution");
        
        // Execute workflow with timeout
        let workflow_future = self.execute_workflow_tasks(&context, &workflow);
        let timeout_future = timeout(self.performance_config.workflow_timeout, workflow_future);
        
        let result = timeout_future.await
            .map_err(|_| format!("Workflow timed out after {:?}", self.performance_config.workflow_timeout))?;
        
        let execution_time = start_time.elapsed();
        
        // Record performance metrics
        self.performance_monitor.record_execution_time(execution_time);
        
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
            "Full workflow execution completed in {:?} (efficiency: {:.2}%)",
            execution_time,
            parallel_efficiency * 100.0
        ));
        
        // Build result
        Ok(WorkflowResult {
            success: result.success,
            workflow_name: context_guard.workflow_name.clone(),
            task_count: context_guard.tasks.len(),
            execution_id: context_guard.execution_id.clone(),
            tasks: context_guard.tasks.clone(),
            errors: result.errors,
            output_logs: context_guard.output_logs.clone(),
            state: result.state,
            execution_time,
            parallel_efficiency,
            metrics: self.performance_monitor.get_metrics(),
        })
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
            
            // Execute tasks in parallel
            let mut task_futures = Vec::new();
            for task in ready_tasks {
                let context_clone = context.clone();
                let task_clone = task.clone();
                let future = self.execute_task_with_retry(context_clone, task_clone);
                task_futures.push(future);
            }
            
            let results = join_all(task_futures).await;
            
            for (i, result) in results.into_iter().enumerate() {
                let task = &ready_tasks[i];
                
                match result {
                    Ok(TaskResult::Completed(_)) => {
                        context.lock().unwrap().end_task(&task.id);
                        completed_tasks.insert(task.id.clone());
                        remaining_tasks.retain(|t| t.id != task.id);
                    }
                    Ok(TaskResult::Paused { task_id, reason }) => {
                        // Workflow is paused
                        let context_guard = context.lock().unwrap();
                        return Ok(WorkflowResult {
                            success: false,
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
                            execution_time: Duration::from_secs(0),
                            parallel_efficiency: 0.0,
                            metrics: self.performance_monitor.get_metrics(),
                        });
                    }
                    Ok(TaskResult::Failed(error)) => {
                        context.lock().unwrap().fail_task(&task.id, &error);
                        return Err(format!("Task {} failed: {}", task.id, error));
                    }
                    Err(error) => {
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
            execution_time: Duration::from_secs(0),
            parallel_efficiency: 0.0,
            metrics: self.performance_monitor.get_metrics(),
        })
    }
    
    async fn execute_task_with_retry(
        &self,
        context: Arc<Mutex<ExecutionContext>>,
        task: Task,
    ) -> Result<TaskResult, String> {
        let retry_config = task.retry_config.clone().unwrap_or_default();
        let mut attempt = 0;
        let mut delay = retry_config.retry_delay;
        
        loop {
            attempt += 1;
            
            // Execute task
            let result = self.execute_task_once(&context, &task).await;
            
            match result {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Check if we should retry
                    if attempt >= retry_config.max_retries {
                        return Err(error);
                    }
                    
                    if !self.should_retry(&error, &retry_config.retry_on_errors) {
                        return Err(error);
                    }
                    
                    // Log retry attempt
                    context.lock().unwrap().log(&format!(
                        "Task {} failed (attempt {}/{}), retrying in {:?}: {}",
                        task.id, attempt, retry_config.max_retries, delay, error
                    ));
                    
                    // Wait before retry
                    sleep(delay).await;
                    
                    // Calculate next delay with exponential backoff
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * retry_config.backoff_multiplier) as u64
                    ).min(retry_config.max_retry_delay);
                }
            }
        }
    }
    
    async fn execute_task_once(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<TaskResult, String> {
        context.lock().unwrap().start_task(&task.id);
        
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
    
    fn should_retry(&self, error: &str, retry_on_errors: &[String]) -> bool {
        retry_on_errors.iter().any(|retry_error| error.contains(retry_error))
    }
    
    pub fn get_performance_metrics(&self) -> WorkflowMetrics {
        self.performance_monitor.get_metrics()
    }
}

// Example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a comprehensive workflow
    let workflow = Workflow {
        id: "full_featured_workflow".to_string(),
        name: "Full Featured Workflow".to_string(),
        tasks: vec![
            Task {
                id: "task1".to_string(),
                name: "Initial Task".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Starting full-featured workflow".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
                retry_config: Some(RetryConfig::default()),
                timeout: Some(Duration::from_secs(30)),
            },
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
                retry_config: Some(RetryConfig::default()),
                timeout: Some(Duration::from_secs(60)),
            },
            Task {
                id: "task3".to_string(),
                name: "Parallel Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Processing data".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
                retry_config: Some(RetryConfig::default()),
                timeout: Some(Duration::from_secs(30)),
            },
            Task {
                id: "task4".to_string(),
                name: "Pause Task".to_string(),
                dependencies: vec!["task2".to_string(), "task3".to_string()],
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
                retry_config: Some(RetryConfig::default()),
                timeout: Some(Duration::from_secs(30)),
            },
            Task {
                id: "task5".to_string(),
                name: "Final Task".to_string(),
                dependencies: vec!["task4".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Workflow completed successfully".to_string()],
                },
                status: TaskStatus::Pending,
                pause_config: None,
                retry_config: Some(RetryConfig::default()),
                timeout: Some(Duration::from_secs(30)),
            },
        ],
    };
    
    // Create engine with full configuration
    let config = PerformanceConfig::default();
    let engine = FullWorkflowEngine::new(config);
    let result = engine.execute_workflow(workflow).await?;
    
    println!("Full-featured workflow executed!");
    println!("Success: {}", result.success);
    println!("Execution ID: {}", result.execution_id);
    println!("Task count: {}", result.task_count);
    println!("Execution time: {:?}", result.execution_time);
    println!("Parallel efficiency: {:.2}%", result.parallel_efficiency * 100.0);
    
    println!("\nPerformance metrics:");
    println!("  Total executions: {}", result.metrics.total_executions);
    println!("  Successful executions: {}", result.metrics.successful_executions);
    println!("  Failed executions: {}", result.metrics.failed_executions);
    println!("  Average execution time: {:?}", result.metrics.average_execution_time);
    println!("  Cache hit rate: {:.2}%", result.metrics.cache_hit_rate * 100.0);
    
    println!("\nTask execution details:");
    for task in result.tasks {
        println!("  {}: {:?} (retries: {})", 
            task.name, 
            task.execution_time,
            task.retry_count
        );
        if let Some(error) = task.error_message {
            println!("    Error: {}", error);
        }
    }
    
    if matches!(result.state, WorkflowState::Paused { .. }) {
        println!("\nWorkflow is paused for user input!");
        println!("State: {:?}", result.state);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_workflow_execution() {
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
                    retry_config: Some(RetryConfig::default()),
                    timeout: Some(Duration::from_secs(30)),
                },
            ],
        };
        
        let config = PerformanceConfig::default();
        let engine = FullWorkflowEngine::new(config);
        let result = engine.execute_workflow(workflow).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.task_count, 1);
        assert!(result.parallel_efficiency >= 0.0);
    }
}
