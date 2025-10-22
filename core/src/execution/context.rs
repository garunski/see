#![allow(clippy::result_large_err)]
use crate::{errors::CoreError, OutputCallback, TaskInfo, TaskStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct ExecutionContext {
    current_task_id: Option<String>,
    per_task_logs: HashMap<String, Vec<String>>,
    output_logs: Vec<String>,
    tasks: Vec<TaskInfo>,
    output_callback: Option<OutputCallback>,
    audit_store: Option<Arc<dyn crate::AuditStore>>,
    execution_id: String,
    workflow_name: String,
    task_start_times: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(
        tasks: Vec<TaskInfo>,
        output_callback: Option<OutputCallback>,
        audit_store: Option<Arc<dyn crate::AuditStore>>,
        execution_id: String,
        workflow_name: String,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            current_task_id: None,
            per_task_logs: HashMap::new(),
            output_logs: Vec::new(),
            tasks,
            output_callback,
            audit_store,
            execution_id,
            workflow_name,
            task_start_times: HashMap::new(),
        }))
    }

    pub fn log(&mut self, msg: &str) {
        self.output_logs.push(msg.to_string());

        if let Some(ref task_id) = self.current_task_id {
            self.per_task_logs
                .entry(task_id.clone())
                .or_default()
                .push(msg.to_string());
        }

        if let Some(ref callback) = self.output_callback {
            callback(msg.to_string());
        }
    }

    pub fn start_task(&mut self, task_id: &str) {
        self.current_task_id = Some(task_id.to_string());
        self.task_start_times
            .insert(task_id.to_string(), chrono::Utc::now().to_rfc3339());
        self.update_task_status(task_id, TaskStatus::InProgress);
    }

    pub fn end_task(&mut self, _task_id: &str) {
        self.current_task_id = None;
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
        }
    }

    pub fn extract_data(self) -> (Vec<String>, HashMap<String, Vec<String>>, Vec<TaskInfo>) {
        (self.output_logs, self.per_task_logs, self.tasks)
    }

    pub fn get_per_task_logs(&self) -> HashMap<String, Vec<String>> {
        self.per_task_logs.clone()
    }

    pub fn get_output_logs(&self) -> Vec<String> {
        self.output_logs.clone()
    }

    pub fn get_tasks(&self) -> Vec<TaskInfo> {
        self.tasks.clone()
    }

    pub fn get_execution_id(&self) -> String {
        self.execution_id.clone()
    }

    pub fn get_workflow_name(&self) -> String {
        self.workflow_name.clone()
    }

    pub fn get_task_logs(&self, task_id: &str) -> Vec<String> {
        self.per_task_logs.get(task_id).cloned().unwrap_or_default()
    }

    pub fn get_task_start_time(&self, task_id: &str) -> String {
        self.task_start_times
            .get(task_id)
            .cloned()
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
    }

    pub fn get_store(&self) -> Option<Arc<dyn crate::AuditStore>> {
        self.audit_store.clone()
    }
}

/// Extension trait for Arc<Mutex<ExecutionContext>> to provide safe methods
pub trait ExecutionContextSafe {
    /// Safe logging method that handles mutex lock errors
    fn safe_log(&self, msg: &str) -> Result<(), CoreError>;

    /// Safe task status update method that handles mutex lock errors
    fn safe_update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<(), CoreError>;

    /// Lock with retry logic for high-contention scenarios
    fn lock_with_retry<F, R>(&self, operation: F, max_retries: usize) -> Result<R, CoreError>
    where
        F: Fn(&MutexGuard<ExecutionContext>) -> Result<R, CoreError>;
}

impl ExecutionContextSafe for Arc<Mutex<ExecutionContext>> {
    fn safe_log(&self, msg: &str) -> Result<(), CoreError> {
        let mut ctx = self
            .lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?;
        ctx.log(msg);
        Ok(())
    }

    fn safe_update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<(), CoreError> {
        let mut ctx = self
            .lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock context: {}", e)))?;
        ctx.update_task_status(task_id, status);
        Ok(())
    }

    fn lock_with_retry<F, R>(&self, operation: F, max_retries: usize) -> Result<R, CoreError>
    where
        F: Fn(&MutexGuard<ExecutionContext>) -> Result<R, CoreError>,
    {
        for attempt in 0..max_retries {
            match self.lock() {
                Ok(guard) => return operation(&guard),
                Err(e) if attempt == max_retries - 1 => {
                    return Err(CoreError::MutexLock(format!(
                        "Failed to acquire lock after {} attempts: {}",
                        max_retries, e
                    )));
                }
                Err(_) => {
                    // Brief backoff before retry
                    std::thread::sleep(std::time::Duration::from_millis(10 * (attempt + 1) as u64));
                }
            }
        }
        unreachable!()
    }
}
