// Execution conversions ONLY

use engine::WorkflowResult as EngineWorkflowResult;
use persistence::{WorkflowExecution, WorkflowStatus};

/// Convert WorkflowResult to WorkflowExecution
pub fn workflow_result_to_execution(
    result: EngineWorkflowResult,
    execution_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
) -> WorkflowExecution {
    let now = chrono::Utc::now();

    // Convert tasks
    let task_executions = result
        .tasks
        .iter()
        .map(|task| {
            crate::bridge::task::task_info_to_execution(
                task,
                &execution_id,
                &result.per_task_logs,
                &result.errors,
                created_at,
                now,
            )
        })
        .collect();

    let now = chrono::Utc::now();

    WorkflowExecution {
        id: execution_id,
        workflow_name: result.workflow_name,
        workflow_snapshot: serde_json::json!({
            "tasks": []
        }), // Empty structure - will be set by caller with actual snapshot
        status: if result.success {
            WorkflowStatus::Complete
        } else {
            WorkflowStatus::Failed
        },
        created_at,
        completed_at: Some(now),
        success: Some(result.success),
        tasks: task_executions,
        timestamp: now,
        audit_trail: Vec::new(), // Will be populated separately
        per_task_logs: result.per_task_logs,
        errors: result.errors,
    }
}
