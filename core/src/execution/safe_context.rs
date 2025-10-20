use crate::execution::context::ExecutionContext;
use crate::TaskStatus;
use std::sync::{Arc, Mutex};

/// Safe wrapper around ExecutionContext that encapsulates mutex operations
pub struct SafeExecutionContext {
    inner: Arc<Mutex<ExecutionContext>>,
}

impl SafeExecutionContext {
    pub fn new(inner: Arc<Mutex<ExecutionContext>>) -> Self {
        Self { inner }
    }

    pub fn log(&self, msg: &str) -> Result<(), String> {
        self.inner.lock().map_err(|e| e.to_string())?.log(msg);
        Ok(())
    }

    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<(), String> {
        self.inner
            .lock()
            .map_err(|e| e.to_string())?
            .update_task_status(task_id, status);
        Ok(())
    }

    pub fn get_inner(&self) -> Arc<Mutex<ExecutionContext>> {
        self.inner.clone()
    }
}
