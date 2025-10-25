use crate::errors::CoreError;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::{error, trace, debug, Instrument};

#[allow(async_fn_in_trait)]
pub trait TaskExecutor: Send + Sync {
    async fn execute(
        &self,
        task_config: &Value,
        logger: &dyn TaskLogger,
    ) -> Result<Value, CoreError>;
}

pub trait TaskLogger: Send + Sync {
    fn log(&self, message: &str);
    fn start_task(&self, task_id: &str);
    fn end_task(&self, task_id: &str);
}

pub struct ContextTaskLogger {
    context: Arc<std::sync::Mutex<crate::execution::context::ExecutionContext>>,
}

impl ContextTaskLogger {
    pub fn new(
        context: Arc<std::sync::Mutex<crate::execution::context::ExecutionContext>>,
    ) -> Self {
        Self { context }
    }
}

impl TaskLogger for ContextTaskLogger {
    fn log(&self, message: &str) {
        match self.context.lock() {
            Ok(mut ctx) => ctx.log(message),
            Err(e) => {
                eprintln!("Failed to lock context for logging: {}", e);
                eprintln!("Message: {}", message);
            }
        }
    }

    fn start_task(&self, task_id: &str) {
        match self.context.lock() {
            Ok(mut ctx) => ctx.start_task(task_id),
            Err(e) => {
                eprintln!("Failed to lock context for task start: {}", e);
                eprintln!("Task ID: {}", task_id);
            }
        }
    }

    fn end_task(&self, task_id: &str) {
        match self.context.lock() {
            Ok(mut ctx) => ctx.end_task(task_id),
            Err(e) => {
                eprintln!("Failed to lock context for task end: {}", e);
                eprintln!("Task ID: {}", task_id);
            }
        }
    }
}

pub struct TaskPersistenceHelper {
    context: Arc<Mutex<crate::execution::context::ExecutionContext>>,
}

impl TaskPersistenceHelper {
    pub fn new(context: Arc<Mutex<crate::execution::context::ExecutionContext>>) -> Self {
        Self { context }
    }

    /// Save task state (start/failed/complete) to database asynchronously
    pub fn save_task_state_async(&self, task_id: &str, status: &str) {
        let Ok(ctx) = self.context.lock() else {
            error!("Failed to lock context for task persistence");
            return;
        };

        let Some(store) = ctx.get_store() else {
            debug!("No store configured, skipping task persistence");
            return;
        };

        let status_str = status.to_string();
        let workflow_id = ctx.get_execution_id().to_string();
        let _workflow_name = ctx.get_workflow_name().to_string();
        let task_name = ctx.get_tasks().iter()
            .find(|t| t.id == task_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "Unknown Task".to_string());
        let logs = ctx.get_task_logs(task_id);
        let _execution_order = ctx.get_tasks().iter()
            .position(|t| t.id == task_id)
            .unwrap_or(0) as i32;
        
        drop(ctx);
        let task_id = task_id.to_string(); // Clone for the async block
        let span =
            tracing::debug_span!("save_task_state_bg", task_id = %task_id, status = %status_str);
        tokio::spawn(
            async move {
                trace!("Saving task state to database");
                
                // Create task execution
                let mut task_exec = persistence::TaskExecution::new(workflow_id, task_id.clone(), task_name);
                task_exec.status = status_str.clone();
                task_exec.logs = logs;
                
                // Set timestamps based on status
                match status_str.as_str() {
                    "in-progress" => task_exec.mark_started(),
                    "complete" => task_exec.mark_completed(true, None, None),
                    "failed" => task_exec.mark_completed(false, None, Some("Task failed".to_string())),
                    _ => {}
                }
                
                // Save to database
                if let Err(e) = store.save_task_execution(task_exec).await {
                    error!("Failed to save task execution: {}", e);
                } else {
                    debug!("Task execution saved successfully");
                }
            }
            .instrument(span),
        );
    }
}
