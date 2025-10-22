use crate::components::ui::BadgeVariant;
use see_core::{AuditEntry, TaskStatus, WorkflowExecution};

pub fn get_status_badge_variant(success: bool) -> BadgeVariant {
    if success {
        BadgeVariant::Success
    } else {
        BadgeVariant::Error
    }
}

pub fn get_task_status_badge_variant(status: &TaskStatus) -> BadgeVariant {
    match status {
        TaskStatus::Complete => BadgeVariant::Success,
        TaskStatus::Failed => BadgeVariant::Error,
        TaskStatus::InProgress => BadgeVariant::Info,
        TaskStatus::Pending => BadgeVariant::Neutral,
    }
}

pub fn format_task_status(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Complete => "Complete",
        TaskStatus::Failed => "Failed",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Pending => "Pending",
    }
}

pub fn get_current_task_audit(
    execution: &WorkflowExecution,
    current_task_id: Option<&String>,
) -> Vec<AuditEntry> {
    if let Some(task_id) = current_task_id {
        execution
            .audit_trail
            .iter()
            .filter(|entry| &entry.task_id == task_id)
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

pub fn get_current_task_logs(
    execution: &WorkflowExecution,
    current_task_id: Option<&String>,
) -> Vec<String> {
    if let Some(task_id) = current_task_id {
        execution
            .per_task_logs
            .get(task_id)
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}
