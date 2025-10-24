// Basic Custom Workflow Engine Implementation
// Phase 1: Sequential Execution

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use serde_json::Value;

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
pub enum TaskFunction {
    CliCommand { command: String, args: Vec<String> },
    CursorAgent { prompt: String, config: Value },
    Custom { name: String, input: Value },
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub function: TaskFunction,
    pub status: TaskStatus,
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
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}

// Execution context
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_name: String,
    pub current_task_id: Option<String>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub output_logs: Vec<String>,
    pub tasks: Vec<TaskInfo>,
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
        }
    }

    pub fn log(&mut self, message: &str) {
        let log_entry = format!("[{}] {}", 
            chrono::Utc::now().format("%H:%M:%S%.3f"), 
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

    pub fn fail_task(&mut self, task_id: &str, error: &str) {
        self.log(&format!("Failed task: {} - {}", task_id, error));
        self.current_task_id = None;
        
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Failed;
        }
    }
}

// Task handler trait
pub trait TaskHandler {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<(), String>;
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
    ) -> Result<(), String> {
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
                return Err(format!("Command failed with exit code: {:?}", output.status.code()));
            }
            
            Ok(())
        } else {
            Err("Invalid task function type".to_string())
        }
    }
}

// Cursor Agent Handler (placeholder)
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
    ) -> Result<(), String> {
        if let TaskFunction::CursorAgent { prompt, .. } = &task.function {
            context.lock().unwrap().log(&format!("Cursor Agent prompt: {}", prompt));
            // Simulate cursor agent execution
            sleep(Duration::from_millis(100)).await;
            Ok(())
        } else {
            Err("Invalid task function type".to_string())
        }
    }
}

// Dependency graph
pub struct DependencyGraph {
    tasks: HashMap<String, Task>,
    dependencies: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new(tasks: Vec<Task>) -> Self {
        let mut graph = Self {
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
        };
        
        for task in tasks {
            let task_id = task.id.clone();
            graph.tasks.insert(task_id.clone(), task);
            
            // Build dependency relationships
            if let Some(task_ref) = graph.tasks.get(&task_id) {
                for dep in &task_ref.dependencies {
                    graph.dependencies.entry(task_id.clone())
                        .or_insert_with(Vec::new)
                        .push(dep.clone());
                }
            }
        }
        
        graph
    }
    
    pub fn get_ready_tasks(
        &self,
        completed: &HashSet<String>,
        running: &HashSet<String>,
    ) -> Vec<Task> {
        self.tasks.values()
            .filter(|task| {
                !completed.contains(&task.id) && !running.contains(&task.id)
            })
            .filter(|task| {
                task.dependencies.iter().all(|dep| completed.contains(dep))
            })
            .cloned()
            .collect()
    }
    
    pub fn has_circular_dependency(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for task_id in self.tasks.keys() {
            if self.dfs_has_cycle(task_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        
        false
    }
    
    fn dfs_has_cycle(
        &self,
        task_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(task_id) {
            return true;
        }
        
        if visited.contains(task_id) {
            return false;
        }
        
        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());
        
        if let Some(deps) = self.dependencies.get(task_id) {
            for dep in deps {
                if self.dfs_has_cycle(dep, visited, rec_stack) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(task_id);
        false
    }
}

// Main workflow engine
pub struct CustomWorkflowEngine {
    task_handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>>,
}

impl CustomWorkflowEngine {
    pub fn new() -> Self {
        let mut handlers: HashMap<String, Box<dyn TaskHandler + Send + Sync>> = HashMap::new();
        
        handlers.insert("cli_command".to_string(), Box::new(CliCommandHandler::new()));
        handlers.insert("cursor_agent".to_string(), Box::new(CursorAgentHandler::new()));
        
        Self { task_handlers: handlers }
    }
    
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, String> {
        let execution_id = Uuid::new_v4().to_string();
        let context = Arc::new(Mutex::new(ExecutionContext::new(&workflow, execution_id.clone())));
        
        context.lock().unwrap().log("Starting workflow execution");
        
        // Build dependency graph
        let dependency_graph = DependencyGraph::new(workflow.tasks.clone());
        
        // Check for circular dependencies
        if dependency_graph.has_circular_dependency() {
            return Err("Circular dependency detected in workflow".to_string());
        }
        
        // Execute tasks in dependency order
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks = workflow.tasks.clone();
        
        while !remaining_tasks.is_empty() {
            let ready_tasks = dependency_graph.get_ready_tasks(&completed_tasks, &HashSet::new());
            
            if ready_tasks.is_empty() {
                return Err("No ready tasks found - possible circular dependency".to_string());
            }
            
            for task in ready_tasks {
                context.lock().unwrap().start_task(&task.id);
                
                let result = self.execute_task(&context, &task).await;
                
                match result {
                    Ok(_) => {
                        context.lock().unwrap().end_task(&task.id);
                        completed_tasks.insert(task.id.clone());
                        remaining_tasks.retain(|t| t.id != task.id);
                    }
                    Err(error) => {
                        context.lock().unwrap().fail_task(&task.id, &error);
                        return Err(format!("Task {} failed: {}", task.id, error));
                    }
                }
            }
        }
        
        context.lock().unwrap().log("Workflow execution completed");
        
        // Build result
        let context_guard = context.lock().unwrap();
        Ok(WorkflowResult {
            success: true,
            workflow_name: context_guard.workflow_name.clone(),
            task_count: context_guard.tasks.len(),
            execution_id: context_guard.execution_id.clone(),
            tasks: context_guard.tasks.clone(),
            errors: vec![],
            output_logs: context_guard.output_logs.clone(),
        })
    }
    
    async fn execute_task(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &Task,
    ) -> Result<(), String> {
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

// Example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test workflow
    let workflow = Workflow {
        id: "test_workflow".to_string(),
        name: "Test Workflow".to_string(),
        tasks: vec![
            Task {
                id: "task1".to_string(),
                name: "First Task".to_string(),
                dependencies: vec![],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Hello, World!".to_string()],
                },
                status: TaskStatus::Pending,
            },
            Task {
                id: "task2".to_string(),
                name: "Second Task".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CliCommand {
                    command: "echo".to_string(),
                    args: vec!["Task 2 completed".to_string()],
                },
                status: TaskStatus::Pending,
            },
            Task {
                id: "task3".to_string(),
                name: "Third Task".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::CursorAgent {
                    prompt: "Generate a simple greeting".to_string(),
                    config: Value::Object(serde_json::Map::new()),
                },
                status: TaskStatus::Pending,
            },
        ],
    };
    
    // Execute workflow
    let engine = CustomWorkflowEngine::new();
    let result = engine.execute_workflow(workflow).await?;
    
    println!("Workflow executed successfully!");
    println!("Execution ID: {}", result.execution_id);
    println!("Task count: {}", result.task_count);
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
    async fn test_basic_workflow_execution() {
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
                },
            ],
        };
        
        let engine = CustomWorkflowEngine::new();
        let result = engine.execute_workflow(workflow).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.task_count, 1);
    }
    
    #[tokio::test]
    async fn test_dependency_resolution() {
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
                },
            ],
        };
        
        let engine = CustomWorkflowEngine::new();
        let result = engine.execute_workflow(workflow).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.task_count, 2);
    }
}
