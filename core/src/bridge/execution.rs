use s_e_e_engine::WorkflowResult as EngineWorkflowResult;
use s_e_e_persistence::{WorkflowExecution, WorkflowExecutionStatus};

pub fn workflow_result_to_execution(
    result: EngineWorkflowResult,
    execution_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
) -> WorkflowExecution {
    let now = chrono::Utc::now();

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
        }),
        status: if result.success {
            WorkflowExecutionStatus::Complete
        } else {
            WorkflowExecutionStatus::Failed
        },
        created_at,
        completed_at: Some(now),
        tasks: task_executions,
        timestamp: now,
        audit_trail: Vec::new(),
        per_task_logs: result.per_task_logs,
        errors: result.errors,
    }
}
