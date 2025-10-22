use see_core::{TaskInfo, WorkflowResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub workflow_file: String,
    pub execution_status: crate::components::ExecutionStatus,
    pub workflow_result: Option<WorkflowResult>,
    pub output_logs: Vec<String>,
    pub current_step: usize,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub tasks: Vec<TaskInfo>,
    pub execution_id: Option<String>,
    pub polling_execution_id: Option<String>,
    pub is_polling: bool,
    pub polling_trigger: usize,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            workflow_file: "workflow.json".to_string(),
            execution_status: crate::components::ExecutionStatus::Idle,
            workflow_result: None,
            output_logs: Vec::new(),
            current_step: 0,
            per_task_logs: HashMap::new(),
            tasks: Vec::new(),
            execution_id: None,
            polling_execution_id: None,
            is_polling: false,
            polling_trigger: 0,
        }
    }
}

impl WorkflowState {
    pub fn reset_before_run(&mut self) {
        self.execution_status = crate::components::ExecutionStatus::Running;
        self.output_logs.clear();
        self.workflow_result = None;
        self.current_step = 0;
        self.per_task_logs.clear();
        self.tasks.clear();
        self.execution_id = None;
    }

    pub fn apply_success(&mut self, result: &WorkflowResult) {
        self.execution_status = crate::components::ExecutionStatus::Complete;
        self.workflow_result = Some(result.clone());
        self.per_task_logs = result.per_task_logs.clone();
        self.tasks = result.tasks.clone();
        self.execution_id = Some(result.execution_id.clone());
    }

    pub fn apply_failure(&mut self, err: &str) {
        self.execution_status = crate::components::ExecutionStatus::Failed;
        self.output_logs.push(format!("Error: {}", err));
    }

    pub fn start_polling(&mut self, execution_id: String) {
        self.polling_execution_id = Some(execution_id);
        self.is_polling = true;
        self.polling_trigger += 1; // Force reactivity
    }

    pub fn stop_polling(&mut self) {
        self.polling_execution_id = None;
        self.is_polling = false;
        self.polling_trigger += 1; // Force reactivity
    }
}
