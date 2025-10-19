use crate::{OutputCallback, TaskInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ExecutionContext {
    current_task_id: Option<String>,
    per_task_logs: HashMap<String, Vec<String>>,
    output_logs: Vec<String>,
    tasks: Vec<TaskInfo>,
    output_callback: Option<OutputCallback>,
}

impl ExecutionContext {
    pub fn new(tasks: Vec<TaskInfo>, output_callback: Option<OutputCallback>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            current_task_id: None,
            per_task_logs: HashMap::new(),
            output_logs: Vec::new(),
            tasks,
            output_callback,
        }))
    }

    pub fn log(&mut self, msg: &str) {
        // Parse task boundary markers
        if msg.starts_with("[TASK_START:") && msg.ends_with("]") {
            let task_id = Self::extract_task_id_from_marker(msg);
            self.current_task_id = Some(task_id.clone());
            self.update_task_status(&task_id, "in-progress");
            return; // Don't add markers to output_logs
        }

        if msg.starts_with("[TASK_END:") && msg.ends_with("]") {
            self.current_task_id = None;
            return; // Don't add markers to output_logs
        }

        // Add to general logs
        self.output_logs.push(msg.to_string());

        // Add to per-task logs if we have a current task
        if let Some(ref task_id) = self.current_task_id {
            self.per_task_logs
                .entry(task_id.clone())
                .or_insert_with(Vec::new)
                .push(msg.to_string());
        }

        // Call output callback if set
        if let Some(ref callback) = self.output_callback {
            callback(msg.to_string());
        }
    }

    fn extract_task_id_from_marker(marker: &str) -> String {
        marker
            .trim()
            .strip_prefix("[TASK_START:")
            .or_else(|| marker.trim().strip_prefix("[TASK_END:"))
            .and_then(|s| s.strip_suffix("]"))
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn update_task_status(&mut self, task_id: &str, status: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status.to_string();
        }
    }

    pub fn extract_data(self) -> (Vec<String>, HashMap<String, Vec<String>>, Vec<TaskInfo>) {
        (self.output_logs, self.per_task_logs, self.tasks)
    }

    // Get snapshots without consuming
    pub fn get_per_task_logs(&self) -> HashMap<String, Vec<String>> {
        self.per_task_logs.clone()
    }

    pub fn get_output_logs(&self) -> Vec<String> {
        self.output_logs.clone()
    }

    pub fn get_tasks(&self) -> Vec<TaskInfo> {
        self.tasks.clone()
    }
}
