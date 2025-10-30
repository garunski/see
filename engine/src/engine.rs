use crate::errors::*;
use crate::handlers::{get_function_type, HandlerRegistry};
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, trace, warn};

pub struct WorkflowEngine {
    handlers: Arc<HandlerRegistry>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(HandlerRegistry::new()),
        }
    }

    fn get_ready_tasks_from_tree(
        &self,
        root_tasks: &[EngineTask],
        completed_tasks: &HashSet<String>,
        waiting_for_input: &HashSet<String>,
    ) -> Vec<EngineTask> {
        let mut ready_tasks = Vec::new();

        fn collect_ready_tasks(
            tasks: &[EngineTask],
            completed_tasks: &HashSet<String>,
            waiting_for_input: &HashSet<String>,
            ready_tasks: &mut Vec<EngineTask>,
        ) {
            for task in tasks {
                if completed_tasks.contains(&task.id) {
                    collect_ready_tasks(
                        &task.next_tasks,
                        completed_tasks,
                        waiting_for_input,
                        ready_tasks,
                    );
                    continue;
                }

                if waiting_for_input.contains(&task.id) {
                    debug!("Task {} skipped - waiting for input", task.id);
                    continue;
                }

                ready_tasks.push(task.clone());
            }
        }

        let root_only_tasks: Vec<EngineTask> =
            root_tasks.iter().filter(|t| t.is_root).cloned().collect();

        collect_ready_tasks(
            &root_only_tasks,
            completed_tasks,
            waiting_for_input,
            &mut ready_tasks,
        );

        trace!(
            ready_count = ready_tasks.len(),
            ready_ids = ?ready_tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
            "Found ready tasks from tree"
        );

        ready_tasks
    }

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

        debug!(execution_id = %execution_id, "Creating execution context");
        let mut context = ExecutionContext::new(execution_id.clone(), workflow.name.clone());

        debug!(execution_id = %execution_id, "Adding tasks to execution context");
        for task in &workflow.tasks {
            context.tasks.insert(task.id.clone(), task.clone());
            trace!(
                execution_id = %execution_id,
                task_id = %task.id,
                task_name = %task.name,
                next_tasks_count = task.next_tasks.len(),
                "Added task to context"
            );
        }

        debug!(execution_id = %execution_id, "Initializing execution state");
        let mut completed_tasks = HashSet::new();
        let mut waiting_for_input = HashSet::new();
        let mut audit_trail = Vec::new();
        let mut errors = Vec::new();
        let mut execution_round = 0;

        trace!(
            execution_id = %execution_id,
            initial_tasks = workflow.tasks.len(),
            "Execution state initialized"
        );

        loop {
            execution_round += 1;

            debug!(
                execution_id = %execution_id,
                round = execution_round,
                completed_count = completed_tasks.len(),
                "Starting execution round"
            );

            trace!(execution_id = %execution_id, "Determining ready tasks");
            let ready_tasks = self.get_ready_tasks_from_tree(
                &workflow.tasks,
                &completed_tasks,
                &waiting_for_input,
            );

            trace!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                completed_count = completed_tasks.len(),
                ready_task_ids = ?ready_tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
                "🔍 Execution round: {} ready tasks found",
                ready_tasks.len()
            );

            if ready_tasks.is_empty() {
                if waiting_for_input.is_empty() {
                    debug!(
                        execution_id = %execution_id,
                        round = execution_round,
                        completed_count = completed_tasks.len(),
                        "No more ready tasks, execution complete"
                    );
                    break;
                } else {
                    debug!(
                        execution_id = %execution_id,
                        round = execution_round,
                        waiting_count = waiting_for_input.len(),
                        "Workflow paused - waiting for {} input(s)",
                        waiting_for_input.len()
                    );
                    break;
                }
            }

            debug!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                "Executing ready tasks in parallel"
            );
            let results = self.execute_round(ready_tasks, &mut context).await?;

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

                if let Some(waiting) = result.output.get("waiting_for_input") {
                    if waiting.as_bool().unwrap_or(false) {
                        waiting_for_input.insert(task.id.clone());
                        debug!(
                            execution_id = %execution_id,
                            task_id = %task.id,
                            task_name = %task.name,
                            "Task waiting for user input"
                        );
                        continue;
                    }
                }

                if result.success {
                    info!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        task_name = %task.name,
                        "✅ Task completed successfully"
                    );

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

                    trace!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        "Task marked as completed"
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

                    audit_trail.push(AuditEntry {
                        task_id: task.id.clone(),
                        status: AuditStatus::Failure,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        changes_count: 0,
                        message: format!("Failed task: {} - {}", task.name, error_msg),
                    });

                    errors.push(format!("Task {}: {}", task.id, error_msg));

                    completed_tasks.insert(task.id.clone());

                    trace!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        "Failed task marked as completed"
                    );
                }
            }

            debug!(
                execution_id = %execution_id,
                completed_count = completed_tasks.len(),
                "📊 Progress: {} completed",
                completed_tasks.len()
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

        let tasks = workflow
            .tasks
            .iter()
            .map(|t| TaskInfo {
                id: t.id.clone(),
                name: t.name.clone(),

                status: if completed_tasks.contains(&t.id) {
                    TaskStatus::Complete
                } else if waiting_for_input.contains(&t.id) {
                    TaskStatus::WaitingForInput
                } else if let Some(context_task) = context.tasks.get(&t.id) {
                    context_task.status.clone()
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

        for task in ready_tasks {
            let task_id = task.id.clone();
            let function_type = get_function_type(&task);

            trace!(
                execution_id = %context.execution_id,
                task_id = %task_id,
                task_name = %task.name,
                function_type = %function_type,
                next_tasks_count = task.next_tasks.len(),
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

        debug!(
            execution_id = %context.execution_id,
            result_count = results.len(),
            "Merging context updates from parallel execution"
        );

        for (task, task_result) in &results {
            let mut logs = Vec::new();

            if let Some(output_str) = task_result.output.as_str() {
                if !output_str.is_empty() {
                    logs.push(format!("Output: {}", output_str));
                }
            } else if !task_result.output.is_null() {
                logs.push(format!("Output: {}", task_result.output));
            }

            if let Some(error) = &task_result.error {
                logs.push(format!("Error: {}", error));
            }

            if !logs.is_empty() {
                context.per_task_logs.insert(task.id.clone(), logs);
            }
        }

        debug!(
            execution_id = %context.execution_id,
            final_result_count = results.len(),
            "Parallel execution round completed"
        );

        Ok(results)
    }

    #[instrument(skip(self), fields(execution_id = %execution_id))]
    pub async fn resume_workflow_execution(
        &self,
        workflow: EngineWorkflow,
        execution_id: String,
        completed_task_ids: HashSet<String>,
        task_user_inputs: HashMap<String, String>,
    ) -> Result<WorkflowResult, EngineError> {
        info!(
            execution_id = %execution_id,
            workflow_name = %workflow.name,
            completed_count = completed_task_ids.len(),
            "🔄 Resuming workflow execution"
        );

        debug!(execution_id = %execution_id, "Creating execution context");
        let mut context = ExecutionContext::new(execution_id.clone(), workflow.name.clone());

        debug!(execution_id = %execution_id, "Adding tasks to execution context");
        for task in &workflow.tasks {
            context.tasks.insert(task.id.clone(), task.clone());
        }

        debug!(
            execution_id = %execution_id,
            initial_completed = completed_task_ids.len(),
            "Initializing execution state with completed tasks"
        );
        let mut completed_tasks = completed_task_ids;

        for (task_id, input_value) in &task_user_inputs {
            completed_tasks.insert(task_id.clone());

            context.log_task(
                task_id.clone(),
                format!("User input provided: {}", input_value),
            );
        }

        let mut waiting_for_input = HashSet::new();
        let mut audit_trail = Vec::new();
        let mut errors = Vec::new();
        let mut execution_round = 0;

        loop {
            execution_round += 1;

            debug!(
                execution_id = %execution_id,
                round = execution_round,
                completed_count = completed_tasks.len(),
                "Starting execution round"
            );

            trace!(execution_id = %execution_id, "Determining ready tasks");
            let ready_tasks = self.get_ready_tasks_from_tree(
                &workflow.tasks,
                &completed_tasks,
                &waiting_for_input,
            );

            trace!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                completed_count = completed_tasks.len(),
                ready_task_ids = ?ready_tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
                "🔍 Execution round: {} ready tasks found",
                ready_tasks.len()
            );

            if ready_tasks.is_empty() {
                if waiting_for_input.is_empty() {
                    debug!(
                        execution_id = %execution_id,
                        round = execution_round,
                        completed_count = completed_tasks.len(),
                        "No more ready tasks, execution complete"
                    );
                    break;
                } else {
                    debug!(
                        execution_id = %execution_id,
                        round = execution_round,
                        waiting_count = waiting_for_input.len(),
                        "Workflow paused - waiting for {} input(s)",
                        waiting_for_input.len()
                    );
                    break;
                }
            }

            debug!(
                execution_id = %execution_id,
                round = execution_round,
                ready_count = ready_tasks.len(),
                "Executing ready tasks in parallel"
            );
            let results = self.execute_round(ready_tasks, &mut context).await?;

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

                if let Some(waiting) = result.output.get("waiting_for_input") {
                    if waiting.as_bool().unwrap_or(false) {
                        waiting_for_input.insert(task.id.clone());
                        debug!(
                            execution_id = %execution_id,
                            task_id = %task.id,
                            task_name = %task.name,
                            "Task waiting for user input"
                        );
                        continue;
                    }
                }

                if result.success {
                    info!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        task_name = %task.name,
                        "✅ Task completed successfully"
                    );

                    audit_trail.push(AuditEntry {
                        task_id: task.id.clone(),
                        status: AuditStatus::Success,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        changes_count: 1,
                        message: format!("Completed task: {}", task.name),
                    });

                    completed_tasks.insert(task.id.clone());
                } else {
                    let error_msg = result.error.unwrap_or_else(|| "Task failed".to_string());
                    error!(
                        execution_id = %execution_id,
                        task_id = %task.id,
                        task_name = %task.name,
                        error = %error_msg,
                        "❌ Task failed"
                    );

                    audit_trail.push(AuditEntry {
                        task_id: task.id.clone(),
                        status: AuditStatus::Failure,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        changes_count: 0,
                        message: format!("Failed task: {} - {}", task.name, error_msg),
                    });

                    errors.push(format!("Task {}: {}", task.id, error_msg));
                    completed_tasks.insert(task.id.clone());
                }
            }

            debug!(
                execution_id = %execution_id,
                completed_count = completed_tasks.len(),
                "📊 Progress: {} completed",
                completed_tasks.len()
            );
        }

        let success = errors.is_empty();

        info!(
            execution_id = %execution_id,
            completed_tasks = completed_tasks.len(),
            total_errors = errors.len(),
            success = success,
            "🏁 Workflow resume execution finished"
        );

        let tasks = workflow
            .tasks
            .iter()
            .map(|t| TaskInfo {
                id: t.id.clone(),
                name: t.name.clone(),
                status: if completed_tasks.contains(&t.id) {
                    TaskStatus::Complete
                } else if waiting_for_input.contains(&t.id) {
                    TaskStatus::WaitingForInput
                } else if let Some(context_task) = context.tasks.get(&t.id) {
                    context_task.status.clone()
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
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}
