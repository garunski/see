use serde_json::Value;
use std::sync::Arc;

/// Trait for executing tasks with clean separation from context management
#[allow(async_fn_in_trait)]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task_config: &Value, logger: &dyn TaskLogger) -> Result<Value, String>;
}

/// Trait for logging task events without direct mutex manipulation
pub trait TaskLogger: Send + Sync {
    fn log(&self, message: &str);
    fn log_task_start(&self, task_id: &str);
    fn log_task_end(&self, task_id: &str);
}

/// Simple implementation of TaskLogger that wraps ExecutionContext
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
        if let Ok(mut ctx) = self.context.lock() {
            ctx.log(message);
        }
    }

    fn log_task_start(&self, task_id: &str) {
        self.log(&format!("[TASK_START:{}]", task_id));
    }

    fn log_task_end(&self, task_id: &str) {
        self.log(&format!("[TASK_END:{}]", task_id));
    }
}
