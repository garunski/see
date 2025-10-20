use see_core::{WorkflowExecution, WorkflowExecutionSummary};

#[derive(Debug, Clone)]
pub struct HistoryState {
    pub workflow_history: Vec<WorkflowExecutionSummary>,
    pub viewing_history_item: Option<WorkflowExecution>,
    pub needs_history_reload: bool,
    pub selected_history_id: Option<String>,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            workflow_history: Vec::new(),
            viewing_history_item: None,
            needs_history_reload: true,
            selected_history_id: None,
        }
    }
}

impl HistoryState {
    pub fn load_execution(&mut self, execution: WorkflowExecution, id: String) {
        self.viewing_history_item = Some(execution);
        self.selected_history_id = Some(id);
        self.needs_history_reload = false;
    }

    pub fn clear_viewing(&mut self) {
        self.viewing_history_item = None;
        self.selected_history_id = None;
    }

    pub fn delete_execution(&mut self, id: &str) {
        self.workflow_history.retain(|item| item.id != id);

        if let Some(ref viewing) = self.viewing_history_item {
            if viewing.id == id {
                self.clear_viewing();
            }
        }

        if self
            .selected_history_id
            .as_ref()
            .is_some_and(|selected_id| selected_id == id)
        {
            self.selected_history_id = None;
        }
    }

    pub fn set_history(&mut self, history: Vec<WorkflowExecutionSummary>) {
        self.workflow_history = history;
        self.needs_history_reload = false;
    }
}
