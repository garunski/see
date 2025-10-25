#![allow(clippy::result_large_err)]
use crate::{
    errors::CoreError,
    types::OutputCallback,
    store::Store,
};
use persistence::TaskInfo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, trace};

pub struct ExecutionContext {
    current_task_id: Option<String>,
    per_task_logs: HashMap<String, Vec<String>>,
    output_logs: Vec<String>,
    tasks: Vec<TaskInfo>,
    output_callback: Option<OutputCallback>,
    store: Option<Arc<dyn Store>>,
    execution_id: String,
    workflow_name: String,
    task_start_times: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(
        tasks: Vec<TaskInfo>,
        output_callback: Option<OutputCallback>,
        store: Option<Arc<dyn Store>>,
        execution_id: String,
        workflow_name: String,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            current_task_id: None,
            per_task_logs: HashMap::new(),
            output_logs: Vec::new(),
            tasks,
            output_callback,
            store,
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
        debug!(
            task_id = %task_id,
            execution_id = %self.execution_id,
            workflow_name = %self.workflow_name,
            "Starting task"
        );
        self.current_task_id = Some(task_id.to_string());
        self.task_start_times
            .insert(task_id.to_string(), chrono::Utc::now().to_rfc3339());
        self.update_task_status(task_id, "in-progress");
    }

    pub fn end_task(&mut self, task_id: &str) {
        debug!(
            task_id = %task_id,
            execution_id = %self.execution_id,
            workflow_name = %self.workflow_name,
            "Ending task"
        );
        self.current_task_id = None;
    }

    pub fn update_task_status(&mut self, task_id: &str, status: &str) {
        trace!(
            task_id = %task_id,
            status = %status,
            execution_id = %self.execution_id,
            workflow_name = %self.workflow_name,
            "Updating task status"
        );
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status.to_string();
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

    // Removed duplicate methods - using the new ones below

    pub fn get_task_logs(&self, task_id: &str) -> Vec<String> {
        self.per_task_logs.get(task_id).cloned().unwrap_or_default()
    }

    pub fn get_task_start_time(&self, task_id: &str) -> String {
        self.task_start_times
            .get(task_id)
            .cloned()
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
    }

    pub fn get_store(&self) -> Option<Arc<dyn Store>> {
        self.store.clone()
    }
    
    pub fn set_store(&mut self, store: Arc<dyn Store>) {
        self.store = Some(store);
    }
    
    pub fn get_execution_id(&self) -> &str {
        &self.execution_id
    }
    
    pub fn get_workflow_name(&self) -> &str {
        &self.workflow_name
    }
    
    pub fn get_tasks(&self) -> &[TaskInfo] {
        &self.tasks
    }

    /// Pause a task for user input
    pub fn pause_for_input(&mut self, task_id: &str, prompt: &str) -> Result<(), CoreError> {
        // Validate task exists
        if !self.tasks.iter().any(|t| t.id == task_id) {
            return Err(CoreError::Validation(format!("Task {} not found", task_id)));
        }

        // Log the pause
        self.log(&format!(
            "⏸️  Task {} paused for user input: {}",
            task_id, prompt
        ));

        // Update task status
        self.update_task_status(task_id, "waiting-for-input");

        // Set current task to None since it's paused
        self.current_task_id = None;

        Ok(())
    }

    /// Resume a paused task
    pub fn resume_task(&mut self, task_id: &str) -> Result<(), CoreError> {
        // Validate task exists and is waiting for input
        let task = self
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| CoreError::Validation(format!("Task {} not found", task_id)))?;

        if task.status != "waiting-for-input" {
            return Err(CoreError::Validation(format!(
                "Task {} is not waiting for input (status: {})",
                task_id, task.status
            )));
        }

        // Log the resume
        self.log(&format!(
            "▶️  Task {} resumed from user input pause",
            task_id
        ));

        // Update task status back to InProgress
        self.update_task_status(task_id, "in-progress");

        // Set as current task
        self.current_task_id = Some(task_id.to_string());

        Ok(())
    }

    /// Check if any task is waiting for input
    pub fn has_waiting_tasks(&self) -> bool {
        self.tasks
            .iter()
            .any(|t| t.status == "waiting-for-input")
    }

    /// Get all tasks waiting for input
    pub fn get_waiting_tasks(&self) -> Vec<&TaskInfo> {
        self.tasks
            .iter()
            .filter(|t| t.status == "waiting-for-input")
            .collect()
    }
}
