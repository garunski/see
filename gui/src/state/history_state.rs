use see_core::WorkflowExecutionSummary;

#[derive(Debug, Clone)]
pub struct HistoryState {
    pub workflow_history: Vec<WorkflowExecutionSummary>,
    pub needs_history_reload: bool,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            workflow_history: Vec::new(),
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
}
