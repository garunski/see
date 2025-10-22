use crate::errors::CoreError;
use crate::execution::context::ExecutionContext;
use crate::TaskStatus;
use std::sync::{Arc, Mutex};

pub struct SafeExecutionContext {
    inner: Arc<Mutex<ExecutionContext>>,
}

impl SafeExecutionContext {
    pub fn new(inner: Arc<Mutex<ExecutionContext>>) -> Self {
        Self { inner }
    }

    #[allow(clippy::result_large_err)]
    pub fn log(&self, msg: &str) -> Result<(), CoreError> {
        match self.inner.lock() {
            Ok(mut ctx) => {
                ctx.log(msg);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to lock context for logging: {}", e);
                Ok(())
            }
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<(), CoreError> {
        match self.inner.lock() {
            Ok(mut ctx) => {
                ctx.update_task_status(task_id, status);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to lock context for task status update: {}", e);
                Ok(())
            }
        }
    }

    pub fn get_inner(&self) -> Arc<Mutex<ExecutionContext>> {
        self.inner.clone()
    }
}
