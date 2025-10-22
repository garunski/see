use see_core::{WorkflowExecutionSummary, WorkflowMetadata};

#[derive(Debug, Clone)]
pub struct HistoryState {
    pub workflow_history: Vec<WorkflowExecutionSummary>,
    pub running_workflows: Vec<WorkflowMetadata>,
    pub needs_history_reload: bool,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            workflow_history: Vec::new(),
            running_workflows: Vec::new(),
            needs_history_reload: true,
        }
    }
}

impl HistoryState {
    pub fn clear_viewing(&mut self) {
        // No longer needed as viewing is handled by routing
    }

    pub fn delete_execution(&mut self, id: &str) {
        self.workflow_history.retain(|item| item.id != id);
    }

    pub fn set_history(&mut self, history: Vec<WorkflowExecutionSummary>) {
        self.workflow_history = history;
        self.needs_history_reload = false;
    }

    pub fn set_running_workflows(&mut self, running: Vec<WorkflowMetadata>) {
        self.running_workflows = running;
    }

    pub fn add_running_workflow(&mut self, workflow: WorkflowMetadata) {
        // Remove if already exists (in case of restart)
        self.running_workflows.retain(|w| w.id != workflow.id);
        self.running_workflows.push(workflow);
    }

    pub fn remove_running_workflow(&mut self, id: &str) {
        self.running_workflows.retain(|w| w.id != id);
    }

    pub fn update_running_workflow(
        &mut self,
        id: &str,
        status: see_core::persistence::models::WorkflowStatus,
    ) {
        if let Some(workflow) = self.running_workflows.iter_mut().find(|w| w.id == id) {
            workflow.status = status;
        }
    }
}
