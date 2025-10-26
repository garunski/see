use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

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
    pub fn clear_viewing(&mut self) {}

    pub fn delete_execution(&mut self, id: &str) {
        self.workflow_history.retain(|item| item.id != id);
    }

    pub fn set_history(&mut self, history: Vec<WorkflowExecutionSummary>) {
        self.workflow_history = history;
        self.needs_history_reload = false;
    }

    pub fn set_running_workflows(&mut self, running: Vec<WorkflowMetadata>) {
        tracing::debug!(
            count = running.len(),
            ids = ?running.iter().map(|w| &w.id).collect::<Vec<_>>(),
            "setting running workflows in state"
        );
        self.running_workflows = running;
    }

    pub fn remove_running_workflow(&mut self, id: &str) {
        self.running_workflows.retain(|w| w.id != id);
    }
}
