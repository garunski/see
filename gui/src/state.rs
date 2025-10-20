use see_core::{
    AuditStore, RedbStore, TaskInfo, WorkflowExecution, WorkflowExecutionSummary, WorkflowResult,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarTab {
    Upload,
    History,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub workflow_file: String,
    pub execution_status: crate::components::ExecutionStatus,
    pub workflow_result: Option<WorkflowResult>,
    pub output_logs: Vec<String>,
    pub show_logs: bool,
    pub show_context: bool,
    pub toast_message: Option<String>,
    pub is_picking_file: bool,
    pub current_step: usize,
    pub per_task_logs: std::collections::HashMap<String, Vec<String>>,
    pub tasks: Vec<TaskInfo>,
    pub execution_id: Option<String>,
    pub workflow_history: Vec<WorkflowExecutionSummary>,
    pub viewing_history_item: Option<WorkflowExecution>,
    pub sidebar_tab: SidebarTab,
    pub needs_history_reload: bool,
    pub selected_history_id: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            workflow_file: "workflow.json".to_string(),
            execution_status: crate::components::ExecutionStatus::Idle,
            workflow_result: None,
            output_logs: Vec::new(),
            show_logs: true,
            show_context: true,
            toast_message: None,
            is_picking_file: false,
            current_step: 0,
            per_task_logs: std::collections::HashMap::new(),
            tasks: Vec::new(),
            execution_id: None,
            workflow_history: Vec::new(),
            viewing_history_item: None,
            sidebar_tab: SidebarTab::Upload,
            needs_history_reload: true,
            selected_history_id: None,
        }
    }
}

impl AppState {
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
        self.toast_message = Some("Workflow completed successfully!".to_string());
        self.needs_history_reload = true;
    }

    pub fn apply_failure(&mut self, err: &str) {
        self.execution_status = crate::components::ExecutionStatus::Failed;
        self.output_logs.push(format!("Error: {}", err));
        self.toast_message = Some(format!("Workflow failed: {}", err));
    }

    pub async fn load_execution(&mut self, id: &str, store: &Arc<RedbStore>) {
        match store.get_workflow_execution(id).await {
            Ok(execution) => {
                self.viewing_history_item = Some(execution);
                self.selected_history_id = Some(id.to_string());
                // Reset current step to 0 when viewing a history item
                self.current_step = 0;
                // Don't switch tabs - stay on history tab
            }
            Err(e) => {
                self.toast_message = Some(format!("Failed to load execution: {}", e));
            }
        }
    }

    pub async fn delete_execution(&mut self, id: &str, store: &Arc<RedbStore>) {
        match store.delete_workflow_execution(id).await {
            Ok(_) => {
                // Remove from history list
                self.workflow_history.retain(|item| item.id != id);

                // If we're viewing this execution, clear the view
                if let Some(ref viewing) = self.viewing_history_item {
                    if viewing.id == id {
                        self.viewing_history_item = None;
                        self.selected_history_id = None;
                    }
                }

                // If this was the selected history item, clear selection
                if self
                    .selected_history_id
                    .as_ref()
                    .map_or(false, |selected_id| selected_id == id)
                {
                    self.selected_history_id = None;
                }

                self.toast_message = Some("Workflow execution deleted".to_string());
            }
            Err(e) => {
                self.toast_message = Some(format!("Failed to delete execution: {}", e));
            }
        }
    }
}
