use crate::errors::CoreError;
use crate::types::TaskStatus;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::{error, trace, Instrument};

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
    pub fn save_task_state_async(&self, task_id: &str, status: TaskStatus) {
        let Ok(ctx) = self.context.lock() else {
            error!("Failed to lock context for task persistence");
            return;
        };

        let Some(store) = ctx.get_store() else {
            return; // No store configured
        };

        let status_clone = status.clone();
        let task_exec = crate::persistence::models::TaskExecution {
            execution_id: ctx.get_execution_id(),
            task_id: task_id.to_string(),
            task_name: task_id.to_string(),
            status,
            logs: ctx.get_task_logs(task_id),
            start_timestamp: ctx.get_task_start_time(task_id),
            end_timestamp: if status_clone == TaskStatus::InProgress {
                String::new()
            } else {
                chrono::Utc::now().to_rfc3339()
            },
        };
        drop(ctx);

        let status_str = status_clone.as_str();
        let span =
            tracing::debug_span!("save_task_state_bg", task_id = %task_id, status = %status_str);
        tokio::spawn(
            async move {
                trace!("Saving task state to database");
                if let Err(e) = store.save_task_execution(&task_exec).await {
                    error!(error = %e, "Failed to save task state");
                }
            }
            .instrument(span),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::context::ExecutionContext;
    use crate::types::{TaskInfo, TaskStatus};

    #[test]
    fn test_task_persistence_helper_creation() {
        // Create test context
        let tasks = vec![TaskInfo {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            status: TaskStatus::Pending,
        }];

        let context = ExecutionContext::new(
            tasks,
            None,
            None,
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        // Create helper - should not panic
        let _helper = TaskPersistenceHelper::new(context);

        // Verify helper can be created (no panic means success)
    }

    #[tokio::test]
    async fn test_save_task_state_without_store() {
        // Test that save_task_state_async handles missing store gracefully
        let tasks = vec![TaskInfo {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            status: TaskStatus::Pending,
        }];

        let context = ExecutionContext::new(
            tasks,
            None,
            None, // No store
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        let helper = TaskPersistenceHelper::new(context);

        // Should not panic with no store
        helper.save_task_state_async("test_task", TaskStatus::InProgress);

        // Give async task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[test]
    fn test_save_all_task_statuses() {
        // Test all three status transitions
        let tasks = vec![TaskInfo {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            status: TaskStatus::Pending,
        }];

        let context = ExecutionContext::new(
            tasks,
            None,
            None,
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        let helper = TaskPersistenceHelper::new(context);

        // All should complete without panic
        helper.save_task_state_async("test_task", TaskStatus::InProgress);
        helper.save_task_state_async("test_task", TaskStatus::Complete);
        helper.save_task_state_async("test_task", TaskStatus::Failed);
    }
}
