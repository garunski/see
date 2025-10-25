use crate::errors::CoreError;
use crate::execution::context::{ExecutionContext, ExecutionContextSafe};
use crate::persistence::models::TaskExecution;
use crate::types::{AuditEntry, AuditStatus, TaskInfo, TaskStatus, WorkflowResult};
use crate::AuditStore;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, instrument, trace};
use uuid::Uuid;

use super::handlers::{CliCommandHandler, CursorAgentHandler};

/// Custom task representation compatible with existing workflow JSON format
#[derive(Debug, Clone)]
pub struct CustomTask {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub function: TaskFunction,
    pub status: TaskStatus,
}

/// Task function types supported by the custom engine
#[derive(Debug, Clone)]
pub enum TaskFunction {
    CliCommand { command: String, args: Vec<String> },
    CursorAgent { prompt: String, config: Value },
    Custom { name: String, input: Value },
}

/// Result of task execution
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

/// Custom workflow representation
#[derive(Debug, Clone)]
pub struct CustomWorkflow {
    pub id: String,
    pub name: String,
    pub tasks: Vec<CustomTask>,
}

/// Trait for task handlers in the custom engine
#[async_trait::async_trait]
pub trait TaskHandler: Send + Sync {
    async fn execute(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &CustomTask,
    ) -> Result<TaskResult, CoreError>;
}

/// Dependency graph for task ordering and circular dependency detection
pub struct DependencyGraph {
    tasks: HashMap<String, CustomTask>,
    dependencies: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new(tasks: Vec<CustomTask>) -> Self {
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
                    graph
                        .dependencies
                        .entry(task_id.clone())
                        .or_insert_with(Vec::new)
                        .push(dep.clone());
                }
            }
        }

        graph
    }

    /// Get tasks that are ready to execute (all dependencies completed)
    pub fn get_ready_tasks(
        &self,
        completed: &HashSet<String>,
        running: &HashSet<String>,
    ) -> Vec<CustomTask> {
        self.tasks
            .values()
            .filter(|task| !completed.contains(&task.id) && !running.contains(&task.id))
            .filter(|task| task.dependencies.iter().all(|dep| completed.contains(dep)))
            .cloned()
            .collect()
    }

    /// Check for circular dependencies using DFS
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

    /// Get all tasks in the graph
    pub fn get_all_tasks(&self) -> Vec<CustomTask> {
        self.tasks.values().cloned().collect()
    }
}

/// Main custom workflow engine
pub struct CustomWorkflowEngine {
    task_handlers: HashMap<String, Box<dyn TaskHandler>>,
    store: Arc<dyn AuditStore>,
}

impl CustomWorkflowEngine {
    pub fn new(store: Arc<dyn AuditStore>) -> Self {
        let mut handlers: HashMap<String, Box<dyn TaskHandler>> = HashMap::new();

        // Create a dummy context for handler initialization
        // The real context will be passed during execution
        let dummy_context = ExecutionContext::new(
            vec![],
            None,
            Some(store.clone()),
            "dummy".to_string(),
            "dummy".to_string(),
        );

        // Register real handlers
        handlers.insert(
            "cli_command".to_string(),
            Box::new(CliCommandHandler::new(dummy_context.clone())),
        );
        handlers.insert(
            "cursor_agent".to_string(),
            Box::new(CursorAgentHandler::new(dummy_context)),
        );

        Self {
            task_handlers: handlers,
            store,
        }
    }

    /// Register a custom task handler
    pub fn register_handler(&mut self, name: String, handler: Box<dyn TaskHandler>) {
        self.task_handlers.insert(name, handler);
    }

    /// Execute a workflow using the custom engine
    #[instrument(skip(self, workflow), fields(workflow_id = %workflow.id, workflow_name = %workflow.name))]
    pub async fn execute_workflow(
        &self,
        workflow: CustomWorkflow,
    ) -> Result<WorkflowResult, CoreError> {
        let execution_id = Uuid::new_v4().to_string();

        info!(
            execution_id = %execution_id,
            workflow_name = %workflow.name,
            task_count = workflow.tasks.len(),
            "🚀 Starting custom workflow execution"
        );

        // Log workflow details
        debug!(
            execution_id = %execution_id,
            "📋 Workflow tasks: {}",
            workflow.tasks.iter()
                .map(|t| format!("{} (deps: {:?})", t.id, t.dependencies))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Create execution context
        let tasks: Vec<TaskInfo> = workflow
            .tasks
            .iter()
            .map(|task| TaskInfo {
                id: task.id.clone(),
                name: task.name.clone(),
                status: TaskStatus::Pending,
            })
            .collect();

        let context = ExecutionContext::new(
            tasks.clone(),
            None, // No output callback for now
            Some(self.store.clone()),
            execution_id.clone(),
            workflow.name.clone(),
        );

        // Log workflow start
        if let Err(e) = context.safe_log("🚀 Starting custom workflow execution\n") {
            error!(error = %e, "Failed to log workflow start");
        }

        if let Err(e) = context.safe_log(&format!(
            "📊 Workflow: {} ({} tasks)\n",
            workflow.name,
            workflow.tasks.len()
        )) {
            error!(error = %e, "Failed to log workflow details");
        }

        // Build dependency graph
        let dependency_graph = DependencyGraph::new(workflow.tasks.clone());

        // Check for circular dependencies
        if dependency_graph.has_circular_dependency() {
            let error_msg = "Circular dependency detected in workflow".to_string();
            error!(execution_id = %execution_id, error = %error_msg);
            if let Err(e) = context.safe_log("❌ Circular dependency detected!\n") {
                error!(error = %e, "Failed to log circular dependency error");
            }
            return Err(CoreError::WorkflowExecution(error_msg));
        }

        debug!(
            execution_id = %execution_id,
            "✅ Dependency graph validated - no circular dependencies"
        );

        // Save workflow metadata
        let workflow_start_time = chrono::Utc::now().to_rfc3339();
        let metadata = crate::persistence::models::WorkflowMetadata {
            id: execution_id.clone(),
            workflow_name: workflow.name.clone(),
            start_timestamp: workflow_start_time.clone(),
            end_timestamp: None,
            status: crate::persistence::models::WorkflowStatus::Running,
            task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
            is_paused: false,
            paused_task_id: None,
        };

        if let Err(e) = self.store.save_workflow_metadata(&metadata).await {
            error!(error = %e, "Failed to save workflow metadata");
        }

        // Save initial task executions
        for task in &tasks {
            let task_exec = crate::persistence::models::TaskExecution {
                execution_id: execution_id.clone(),
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                status: TaskStatus::Pending,
                logs: Vec::new(),
                start_timestamp: String::new(),
                end_timestamp: String::new(),
            };
            if let Err(e) = self.store.save_task_execution(&task_exec).await {
                error!(error = %e, task_id = %task.id, "Failed to save initial task execution");
            }
        }

        // Execute tasks in dependency order
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks = dependency_graph.get_all_tasks();
        let mut errors = Vec::new();
        let mut audit_trail = Vec::new();
        let mut per_task_logs = std::collections::HashMap::new();
        let mut execution_round = 0;

        debug!(
            execution_id = %execution_id,
            "🔄 Starting task execution loop with {} tasks",
            remaining_tasks.len()
        );

        while !remaining_tasks.is_empty() {
            execution_round += 1;
            let ready_tasks = dependency_graph.get_ready_tasks(&completed_tasks, &HashSet::new());

            trace!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                completed_count = completed_tasks.len(),
                remaining_count = remaining_tasks.len(),
                "🔍 Execution round: {} ready tasks found",
                ready_tasks.len()
            );

            if ready_tasks.is_empty() {
                let error_msg = "No ready tasks found - possible circular dependency".to_string();
                error!(execution_id = %execution_id, error = %error_msg);
                if let Err(e) =
                    context.safe_log("❌ No ready tasks found - possible circular dependency!\n")
                {
                    error!(error = %e, "Failed to log no ready tasks error");
                }
                return Err(CoreError::WorkflowExecution(error_msg));
            }

            // Execute only the first ready task for sequential execution
            // If multiple tasks are ready (no dependencies), execute them in original order
            let task_to_execute = if ready_tasks.len() > 1 {
                // Find the first ready task that appears earliest in the original workflow
                remaining_tasks
                    .iter()
                    .find(|t| ready_tasks.iter().any(|rt| rt.id == t.id))
                    .cloned()
            } else {
                ready_tasks.first().cloned()
            };

            if let Some(task) = task_to_execute {
                info!(
                    execution_id = %execution_id,
                    task_id = %task.id,
                    task_name = %task.name,
                    round = execution_round,
                    "▶️  Executing task: {}",
                    task.name
                );

                // Start task
                let task_start_log =
                    format!("▶️  Starting task: {} (ID: {})\n", task.name, task.id);
                context.safe_log(&task_start_log)?;
                context.safe_update_task_status(&task.id, TaskStatus::InProgress)?;

                // Initialize task logs
                per_task_logs.insert(task.id.clone(), vec![task_start_log.clone()]);

                // Update task execution in database
                let task_start_time = chrono::Utc::now().to_rfc3339();
                let task_exec = TaskExecution {
                    execution_id: execution_id.clone(),
                    task_id: task.id.clone(),
                    task_name: task.name.clone(),
                    status: TaskStatus::InProgress,
                    logs: Vec::new(),
                    start_timestamp: task_start_time.clone(),
                    end_timestamp: String::new(),
                };
                if let Err(e) = self.store.save_task_execution(&task_exec).await {
                    error!(error = %e, task_id = %task.id, "Failed to save task start");
                }

                // Execute task
                let result = self.execute_task(&context, &task).await;

                // Get logs from context after task execution
                let task_logs = {
                    let ctx = context.lock().map_err(|e| {
                        CoreError::MutexLock(format!("Failed to lock context for logs: {}", e))
                    })?;
                    ctx.get_per_task_logs()
                        .get(&task.id)
                        .cloned()
                        .unwrap_or_default()
                };

                match result {
                    Ok(task_result) => {
                        if task_result.success {
                            info!(
                                execution_id = %execution_id,
                                task_id = %task.id,
                                task_name = %task.name,
                                "✅ Task completed successfully"
                            );

                            let task_complete_log =
                                format!("✅ Completed task: {} (ID: {})\n", task.name, task.id);
                            context.safe_log(&task_complete_log)?;
                            context.safe_update_task_status(&task.id, TaskStatus::Complete)?;

                            // Add completion log to task logs
                            if let Some(logs) = per_task_logs.get_mut(&task.id) {
                                logs.push(task_complete_log);
                            }

                            // Update task execution completion in database
                            let task_end_time = chrono::Utc::now().to_rfc3339();
                            let task_exec = TaskExecution {
                                execution_id: execution_id.clone(),
                                task_id: task.id.clone(),
                                task_name: task.name.clone(),
                                status: TaskStatus::Complete,
                                logs: task_logs.clone(),
                                start_timestamp: task_start_time.clone(),
                                end_timestamp: task_end_time.clone(),
                            };
                            if let Err(e) = self.store.save_task_execution(&task_exec).await {
                                error!(error = %e, task_id = %task.id, "Failed to save task completion");
                            }

                            // Add to audit trail
                            audit_trail.push(AuditEntry {
                                task_id: task.id.clone(),
                                status: AuditStatus::Success,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                changes_count: 1, // Placeholder
                                message: format!("Completed task: {}", task.name),
                            });

                            completed_tasks.insert(task.id.clone());
                            remaining_tasks.retain(|t| t.id != task.id);

                            debug!(
                                execution_id = %execution_id,
                                completed_count = completed_tasks.len(),
                                remaining_count = remaining_tasks.len(),
                                "📊 Progress: {} completed, {} remaining",
                                completed_tasks.len(),
                                remaining_tasks.len()
                            );
                        } else {
                            let error_msg = task_result
                                .error
                                .unwrap_or_else(|| "Task failed".to_string());

                            error!(
                                execution_id = %execution_id,
                                task_id = %task.id,
                                task_name = %task.name,
                                error = %error_msg,
                                "❌ Task failed"
                            );

                            let task_fail_log = format!(
                                "❌ Failed task: {} (ID: {}) - {}\n",
                                task.name, task.id, error_msg
                            );
                            context.safe_log(&task_fail_log)?;
                            context.safe_update_task_status(&task.id, TaskStatus::Failed)?;

                            // Add failure log to task logs
                            if let Some(logs) = per_task_logs.get_mut(&task.id) {
                                logs.push(task_fail_log);
                            }

                            // Update task execution failure in database
                            let task_end_time = chrono::Utc::now().to_rfc3339();
                            let task_exec = TaskExecution {
                                execution_id: execution_id.clone(),
                                task_id: task.id.clone(),
                                task_name: task.name.clone(),
                                status: TaskStatus::Failed,
                                logs: task_logs.clone(),
                                start_timestamp: task_start_time.clone(),
                                end_timestamp: task_end_time.clone(),
                            };
                            if let Err(e) = self.store.save_task_execution(&task_exec).await {
                                error!(error = %e, task_id = %task.id, "Failed to save task failure");
                            }

                            errors.push(format!("Task {}: {}", task.id, error_msg));

                            // Add to audit trail
                            audit_trail.push(AuditEntry {
                                task_id: task.id.clone(),
                                status: AuditStatus::Failure,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                changes_count: 0,
                                message: format!("Failed task: {} - {}", task.name, error_msg),
                            });

                            // For now, continue with other tasks even if one fails
                            completed_tasks.insert(task.id.clone());
                            remaining_tasks.retain(|t| t.id != task.id);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Task {} failed: {}", task.id, e);

                        error!(
                            execution_id = %execution_id,
                            task_id = %task.id,
                            task_name = %task.name,
                            error = %e,
                            "❌ Task execution error"
                        );

                        let task_error_log = format!(
                            "❌ Failed task: {} (ID: {}) - {}\n",
                            task.name, task.id, error_msg
                        );
                        context.safe_log(&task_error_log)?;
                        context.safe_update_task_status(&task.id, TaskStatus::Failed)?;

                        // Add error log to task logs
                        if let Some(logs) = per_task_logs.get_mut(&task.id) {
                            logs.push(task_error_log);
                        }

                        // Update task execution error in database
                        let task_end_time = chrono::Utc::now().to_rfc3339();
                        let task_exec = TaskExecution {
                            execution_id: execution_id.clone(),
                            task_id: task.id.clone(),
                            task_name: task.name.clone(),
                            status: TaskStatus::Failed,
                            logs: task_logs.clone(),
                            start_timestamp: task_start_time.clone(),
                            end_timestamp: task_end_time.clone(),
                        };
                        if let Err(e) = self.store.save_task_execution(&task_exec).await {
                            error!(error = %e, task_id = %task.id, "Failed to save task error");
                        }

                        errors.push(error_msg.clone());

                        // Add to audit trail
                        audit_trail.push(AuditEntry {
                            task_id: task.id.clone(),
                            status: AuditStatus::Failure,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            changes_count: 0,
                            message: format!("Error in task: {} - {}", task.name, error_msg),
                        });

                        // For now, continue with other tasks even if one fails
                        completed_tasks.insert(task.id.clone());
                        remaining_tasks.retain(|t| t.id != task.id);
                    }
                }
            } // End of if let Some(task) = ready_tasks.first()
        }

        context.safe_log("🎉 Custom workflow execution completed\n")?;

        info!(
            execution_id = %execution_id,
            completed_tasks = completed_tasks.len(),
            total_errors = errors.len(),
            "🏁 Workflow execution finished"
        );

        // Update workflow metadata
        let metadata = crate::persistence::models::WorkflowMetadata {
            id: execution_id.clone(),
            workflow_name: workflow.name.clone(),
            start_timestamp: workflow_start_time.clone(),
            end_timestamp: Some(chrono::Utc::now().to_rfc3339()),
            status: if errors.is_empty() {
                crate::persistence::models::WorkflowStatus::Complete
            } else {
                crate::persistence::models::WorkflowStatus::Failed
            },
            task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
            is_paused: false,
            paused_task_id: None,
        };

        if let Err(e) = self.store.save_workflow_metadata(&metadata).await {
            error!(error = %e, "Failed to save workflow completion metadata");
        }

        // Save WorkflowExecution for history display
        let execution = crate::persistence::models::WorkflowExecution {
            id: execution_id.clone(),
            workflow_name: workflow.name.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            success: errors.is_empty(),
            tasks: tasks
                .iter()
                .map(|t| crate::types::TaskInfo {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    status: t.status.clone(),
                })
                .collect(),
            audit_trail: audit_trail.clone(),
            per_task_logs: per_task_logs.clone(),
            errors: errors.clone(),
        };

        if let Err(e) = self.store.save_workflow_execution(&execution).await {
            error!(error = %e, "Failed to save workflow execution for history");
        }

        // Extract context data
        let (output_logs, context_per_task_logs, final_tasks) =
            match Arc::try_unwrap(context.clone()) {
                Ok(context) => context
                    .into_inner()
                    .map_err(|e| {
                        CoreError::ExecutionContext(format!("Failed to unwrap context: {:?}", e))
                    })?
                    .extract_data(),
                Err(context) => {
                    let ctx = context.lock().map_err(|e| {
                        CoreError::MutexLock(format!("Failed to lock context: {}", e))
                    })?;
                    (
                        ctx.get_output_logs(),
                        ctx.get_per_task_logs(),
                        ctx.get_tasks(),
                    )
                }
            };

        // Merge context logs with our custom logs
        for (task_id, context_logs) in context_per_task_logs {
            if let Some(custom_logs) = per_task_logs.get_mut(&task_id) {
                // Prepend context logs to custom logs
                let mut merged_logs = context_logs;
                merged_logs.extend(custom_logs.clone());
                per_task_logs.insert(task_id, merged_logs);
            } else {
                // Use context logs if no custom logs exist
                per_task_logs.insert(task_id, context_logs);
            }
        }

        // Build result
        let result = WorkflowResult {
            success: errors.is_empty(),
            workflow_name: workflow.name.clone(),
            task_count: workflow.tasks.len(),
            execution_id: execution_id.clone(),
            tasks: final_tasks,
            final_context: Value::Object(serde_json::Map::new()), // Placeholder
            audit_trail,
            per_task_logs,
            errors,
            output_logs,
        };

        info!(
            execution_id = %execution_id,
            workflow_name = %workflow.name,
            success = result.success,
            "Custom workflow execution completed"
        );

        Ok(result)
    }

    /// Execute a single task
    #[instrument(skip(self, context, task), fields(task_id = %task.id))]
    async fn execute_task(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        task: &CustomTask,
    ) -> Result<TaskResult, CoreError> {
        let handler_name = match &task.function {
            TaskFunction::CliCommand { .. } => "cli_command",
            TaskFunction::CursorAgent { .. } => "cursor_agent",
            TaskFunction::Custom { name, .. } => name,
        };

        debug!(
            task_id = %task.id,
            task_name = %task.name,
            handler = %handler_name,
            "🔧 Executing task with handler: {}",
            handler_name
        );

        let handler = self.task_handlers.get(handler_name).ok_or_else(|| {
            CoreError::WorkflowExecution(format!(
                "No handler found for task type: {}",
                handler_name
            ))
        })?;

        let start_time = std::time::Instant::now();
        let result = handler.execute(context, task).await;
        let duration = start_time.elapsed();

        match &result {
            Ok(task_result) => {
                debug!(
                    task_id = %task.id,
                    success = task_result.success,
                    duration_ms = duration.as_millis(),
                    "🔧 Task handler completed in {}ms",
                    duration.as_millis()
                );
            }
            Err(e) => {
                error!(
                    task_id = %task.id,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "🔧 Task handler failed after {}ms: {}",
                    duration.as_millis(),
                    e
                );
            }
        }

        result
    }
}

/// Convert JSON workflow to custom workflow format
pub fn convert_workflow_from_json(workflow_data: &str) -> Result<CustomWorkflow, CoreError> {
    let workflow_json: serde_json::Value = serde_json::from_str(workflow_data).map_err(|e| {
        CoreError::WorkflowExecution(format!("Failed to parse workflow JSON: {}", e))
    })?;

    let id = workflow_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let name = workflow_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Workflow")
        .to_string();

    let tasks_array = workflow_json
        .get("tasks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            CoreError::WorkflowExecution("Missing 'tasks' array in workflow".to_string())
        })?;

    let mut custom_tasks = Vec::new();

    for task_json in tasks_array {
        let task_id = task_json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let task_name = task_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Task")
            .to_string();

        // Extract dependencies
        let dependencies = task_json
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        // Extract function configuration
        let function = task_json.get("function").ok_or_else(|| {
            CoreError::WorkflowExecution("Missing 'function' in task".to_string())
        })?;

        let function_type = function
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("custom");

        let custom_task = CustomTask {
            id: task_id,
            name: task_name,
            dependencies,
            function: match function_type {
                "custom" => {
                    let name = function
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let input = function
                        .get("input")
                        .cloned()
                        .unwrap_or(Value::Object(serde_json::Map::new()));

                    match name.as_str() {
                        "cli_command" => {
                            let command = input
                                .get("command")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let args = input
                                .get("args")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .map(String::from)
                                        .collect()
                                })
                                .unwrap_or_default();
                            TaskFunction::CliCommand { command, args }
                        }
                        "cursor_agent" => {
                            let prompt = input
                                .get("prompt")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            TaskFunction::CursorAgent {
                                prompt,
                                config: input,
                            }
                        }
                        _ => TaskFunction::Custom { name, input },
                    }
                }
                _ => {
                    return Err(CoreError::WorkflowExecution(format!(
                        "Unsupported function type: {}",
                        function_type
                    )));
                }
            },
            status: TaskStatus::Pending,
        };

        custom_tasks.push(custom_task);
    }

    Ok(CustomWorkflow {
        id,
        name,
        tasks: custom_tasks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dependency_graph_no_dependencies() {
        let tasks = vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ];

        let graph = DependencyGraph::new(tasks);
        let completed = HashSet::new();
        let running = HashSet::new();

        let ready_tasks = graph.get_ready_tasks(&completed, &running);
        assert_eq!(ready_tasks.len(), 2);
        assert!(!graph.has_circular_dependency());
    }

    #[tokio::test]
    async fn test_dependency_graph_with_dependencies() {
        let tasks = vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec![],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ];

        let graph = DependencyGraph::new(tasks);
        let mut completed = HashSet::new();
        let running = HashSet::new();

        // Initially only task1 should be ready
        let ready_tasks = graph.get_ready_tasks(&completed, &running);
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, "task1");

        // After task1 is completed, task2 should be ready
        completed.insert("task1".to_string());
        let ready_tasks = graph.get_ready_tasks(&completed, &running);
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, "task2");

        assert!(!graph.has_circular_dependency());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let tasks = vec![
            CustomTask {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                dependencies: vec!["task2".to_string()],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
            CustomTask {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                dependencies: vec!["task1".to_string()],
                function: TaskFunction::Custom {
                    name: "test".to_string(),
                    input: Value::Null,
                },
                status: TaskStatus::Pending,
            },
        ];

        let graph = DependencyGraph::new(tasks);
        assert!(graph.has_circular_dependency());
    }

    #[tokio::test]
    async fn test_workflow_conversion() {
        let workflow_json = r#"{
            "id": "test_workflow",
            "name": "Test Workflow",
            "tasks": [
                {
                    "id": "task1",
                    "name": "Task 1",
                    "dependencies": [],
                    "function": {
                        "type": "custom",
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["hello"]
                        }
                    }
                }
            ]
        }"#;

        let custom_workflow = convert_workflow_from_json(workflow_json).unwrap();
        assert_eq!(custom_workflow.name, "Test Workflow");
        assert_eq!(custom_workflow.tasks.len(), 1);

        match &custom_workflow.tasks[0].function {
            TaskFunction::CliCommand { command, args } => {
                assert_eq!(command, "echo");
                assert_eq!(args, &["hello"]);
            }
            _ => panic!("Expected CliCommand function"),
        }
    }
}
