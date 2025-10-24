use s_e_e_core::{TaskInfo, WorkflowResult};
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
        }
    }
}

impl WorkflowState {
    pub fn reset_before_run(&mut self) {
        tracing::info!("Resetting workflow state before execution");
        self.execution_status = crate::components::ExecutionStatus::Running;
        self.output_logs.clear();
        self.workflow_result = None;
        self.current_step = 0;
        self.per_task_logs.clear();
        self.tasks.clear();
        self.execution_id = None;
        tracing::debug!("Workflow state reset completed");
    }
}
