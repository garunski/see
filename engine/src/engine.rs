//! Main workflow execution engine with parallel and sequential task execution

use crate::errors::*;
use crate::graph::DependencyGraph;
use crate::handlers::{get_function_type, HandlerRegistry};
use crate::types::*;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, trace, warn};

/// Main workflow execution engine
pub struct WorkflowEngine {
    handlers: Arc<HandlerRegistry>,
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(HandlerRegistry::new()),
        }
    }

    /// Execute a workflow
    #[instrument(skip(self), fields(workflow_id = %workflow.id, workflow_name = %workflow.name, task_count = workflow.tasks.len()))]
    pub async fn execute_workflow(
        &self,
        workflow: EngineWorkflow,
    ) -> Result<WorkflowResult, EngineError> {
        let execution_id = uuid::Uuid::new_v4().to_string();

        info!(
            execution_id = %execution_id,
            workflow_name = %workflow.name,
            task_count = workflow.tasks.len(),
            "🚀 Starting workflow execution"
        );

        trace!(
            execution_id = %execution_id,
            "Workflow tasks: {:?}",
            workflow.tasks.iter().map(|t| &t.id).collect::<Vec<_>>()
        );

        // Build dependency graph
        debug!(execution_id = %execution_id, "Building dependency graph");
        let graph = DependencyGraph::new(workflow.tasks.clone())?;
        debug!(
            execution_id = %execution_id,
            total_tasks = graph.get_all_tasks().len(),
            "Dependency graph built successfully"
        );

        // Create execution context
        debug!(execution_id = %execution_id, "Creating execution context");
        let mut context = ExecutionContext::new(execution_id.clone(), workflow.name.clone());

        // Add all tasks to context
        debug!(execution_id = %execution_id, "Adding tasks to execution context");
        for task in &workflow.tasks {
            context.tasks.insert(task.id.clone(), task.clone());
            trace!(
                execution_id = %execution_id,
                task_id = %task.id,
                task_name = %task.name,
                dependencies = ?task.dependencies,
                "Added task to context"
            );
        }

        // Track execution state
        debug!(execution_id = %execution_id, "Initializing execution state");
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks = workflow.tasks.clone();
        let mut audit_trail = Vec::new();
        let mut errors = Vec::new();
        let mut execution_round = 0;

        trace!(
            execution_id = %execution_id,
            initial_remaining = remaining_tasks.len(),
            "Execution state initialized"
        );

        // Main execution loop
        while !remaining_tasks.is_empty() {
            execution_round += 1;

            debug!(
                execution_id = %execution_id,
                round = execution_round,
                remaining_count = remaining_tasks.len(),
                completed_count = completed_tasks.len(),
                "Starting execution round"
            );

            // Get ready tasks (all dependencies completed)
            trace!(execution_id = %execution_id, "Determining ready tasks");
            let ready_tasks = graph.get_ready_tasks(&completed_tasks);

            trace!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                completed_count = completed_tasks.len(),
                remaining_count = remaining_tasks.len(),
                ready_task_ids = ?ready_tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
                "🔍 Execution round: {} ready tasks found",
                ready_tasks.len()
            );

            if ready_tasks.is_empty() {
                let error_msg = "No ready tasks found - possible circular dependency or deadlock";
                error!(
                    execution_id = %execution_id,
                    round = execution_round,
                    remaining_tasks = ?remaining_tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
                    completed_tasks = ?completed_tasks.iter().collect::<Vec<_>>(),
                    error = %error_msg
                );
                return Err(EngineError::Execution(error_msg.to_string()));
            }

            // Execute all ready tasks in parallel
            debug!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                "Executing ready tasks in parallel"
            );
            let results = self.execute_round(ready_tasks, &mut context).await?;

            // Process results
            debug!(
                execution_id = %execution_id,
                round = execution_round,
                result_count = results.len(),
                "Processing task execution results"
            );

            for (task, result) in results {
                trace!(
                    execution_id = %execution_id,
                    task_id = %task.id,
                    task_name = %task.name,
                    success = result.success,
                    "Processing task result"
                );

                if result.success {
                    info!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        task_name = %task.name,
                        "✅ Task completed successfully"
                    );

                    // Add to audit trail
                    debug!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        "Adding success to audit trail"
                    );
                    audit_trail.push(AuditEntry {
                        task_id: task.id.clone(),
                        status: AuditStatus::Success,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        changes_count: 1,
                        message: format!("Completed task: {}", task.name),
                    });

                    completed_tasks.insert(task.id.clone());
                    remaining_tasks.retain(|t| t.id != task.id);

                    trace!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        "Task marked as completed and removed from remaining"
                    );
                } else {
                    let error_msg = result.error.unwrap_or_else(|| "Task failed".to_string());
                    error!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        task_name = %task.name,
                        error = %error_msg,
                        "❌ Task failed"
                    );

                    debug!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        "Adding failure to audit trail"
                    );

                    // Add to audit trail
                    audit_trail.push(AuditEntry {
                        task_id: task.id.clone(),
                        status: AuditStatus::Failure,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        changes_count: 0,
                        message: format!("Failed task: {} - {}", task.name, error_msg),
                    });

                    errors.push(format!("Task {}: {}", task.id, error_msg));

                    // For now, continue with other tasks even if one fails
                    completed_tasks.insert(task.id.clone());
                    remaining_tasks.retain(|t| t.id != task.id);

                    trace!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        "Failed task marked as completed and removed from remaining"
                    );
                }
            }

            debug!(
                execution_id = %execution_id,
                completed_count = completed_tasks.len(),
                remaining_count = remaining_tasks.len(),
                "📊 Progress: {} completed, {} remaining",
                completed_tasks.len(),
                remaining_tasks.len()
            );
        }

        let success = errors.is_empty();

        info!(
            execution_id = %execution_id,
            completed_tasks = completed_tasks.len(),
            total_errors = errors.len(),
            success = success,
            "🏁 Workflow execution finished"
        );

        // Build task info for result
        let tasks = workflow
            .tasks
            .iter()
            .map(|t| TaskInfo {
                id: t.id.clone(),
                name: t.name.clone(),
                status: if completed_tasks.contains(&t.id) {
                    TaskStatus::Complete
                } else {
                    TaskStatus::Failed
                },
            })
            .collect();

        Ok(WorkflowResult {
            success,
            workflow_name: workflow.name,
            tasks,
            audit_trail,
            per_task_logs: context.per_task_logs,
            errors,
        })
    }

    /// Execute a round of ready tasks in parallel
    #[instrument(skip(self, context), fields(ready_count = ready_tasks.len()))]
    async fn execute_round(
        &self,
        ready_tasks: Vec<EngineTask>,
        context: &mut ExecutionContext,
    ) -> Result<Vec<(EngineTask, TaskResult)>, EngineError> {
        debug!(
            execution_id = %context.execution_id,
            ready_count = ready_tasks.len(),
            "Starting parallel execution of ready tasks"
        );

        let mut handles = Vec::new();

        // Spawn parallel execution for each ready task
        for task in ready_tasks {
            let task_id = task.id.clone();
            let function_type = get_function_type(&task);

            trace!(
                execution_id = %context.execution_id,
                task_id = %task_id,
                task_name = %task.name,
                function_type = %function_type,
                dependencies = ?task.dependencies,
                "Preparing task for parallel execution"
            );

            info!(
                execution_id = %context.execution_id,
                task_id = %task_id,
                task_name = %task.name,
                function_type = %function_type,
                "▶️  Executing task: {}",
                task.name
            );

            // Create a task for parallel execution
            debug!(
                execution_id = %context.execution_id,
                task_id = %task_id,
                "Spawning async task for parallel execution"
            );

            let task_clone = task.clone();
            let mut context_clone = context.clone();
            let function_type_clone = function_type.to_string();
            let handlers_clone = Arc::clone(&self.handlers);

            let handle = tokio::spawn(async move {
                trace!(
                    execution_id = %context_clone.execution_id,
                    task_id = %task_clone.id,
                    function_type = %function_type_clone,
                    "Starting task execution in async context"
                );

                let handler = match handlers_clone.get_handler(&function_type_clone) {
                    Some(h) => h,
                    None => {
                        warn!(
                            execution_id = %context_clone.execution_id,
                            task_id = %task_clone.id,
                            function_type = %function_type_clone,
                            "No handler found for function type"
                        );
                        return (
                            task_clone,
                            TaskResult {
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(format!(
                                    "No handler found for function type: {}",
                                    function_type_clone
                                )),
                            },
                        );
                    }
                };

                debug!(
                    execution_id = %context_clone.execution_id,
                    task_id = %task_clone.id,
                    "Calling task handler"
                );

                match handler.execute(&mut context_clone, &task_clone).await {
                    Ok(result) => {
                        trace!(
                            execution_id = %context_clone.execution_id,
                            task_id = %task_clone.id,
                            success = result.success,
                            "Task handler completed successfully"
                        );
                        (task_clone, result)
                    }
                    Err(e) => {
                        error!(
                            execution_id = %context_clone.execution_id,
                            task_id = %task_clone.id,
                            error = %e,
                            "Task handler failed"
                        );
                        (
                            task_clone,
                            TaskResult {
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(e.to_string()),
                            },
                        )
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        debug!(
            execution_id = %context.execution_id,
            handle_count = handles.len(),
            "Waiting for all parallel tasks to complete"
        );

        let mut results = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            trace!(
                execution_id = %context.execution_id,
                handle_index = i,
                "Waiting for task handle to complete"
            );

            match handle.await {
                Ok(result) => {
                    trace!(
                        execution_id = %context.execution_id,
                        task_id = %result.0.id,
                        success = result.1.success,
                        "Task handle completed successfully"
                    );
                    results.push(result);
                }
                Err(e) => {
                    error!(
                        execution_id = %context.execution_id,
                        handle_index = i,
                        error = %e,
                        "Task execution panicked"
                    );
                    return Err(EngineError::Execution(format!(
                        "Task execution panicked: {}",
                        e
                    )));
                }
            }
        }

        // Merge context updates back
        debug!(
            execution_id = %context.execution_id,
            result_count = results.len(),
            "Merging context updates from parallel execution"
        );

        for (task, _) in &results {
            if let Some(task_logs) = context.per_task_logs.get(&task.id) {
                trace!(
                    execution_id = %context.execution_id,
                    task_id = %task.id,
                    log_count = task_logs.len(),
                    "Merging task logs back to context"
                );
                context
                    .per_task_logs
                    .insert(task.id.clone(), task_logs.clone());
            }
        }

        debug!(
            execution_id = %context.execution_id,
            final_result_count = results.len(),
            "Parallel execution round completed"
        );

        Ok(results)
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}
