use crate::errors::CoreError;
use serde_json::Value;
use std::sync::Arc;

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
