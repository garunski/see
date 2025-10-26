use crate::services::workflow::run_workflow_by_id;

pub fn use_workflow_execution() -> impl Fn(String, String) + 'static {
    move |workflow_name: String, workflow_id: String| {
        tracing::debug!(
            workflow_name = %workflow_name,
            workflow_id = %workflow_id,
            "User initiated workflow execution"
        );

        tokio::spawn(async move {
            tracing::debug!(
                workflow_name = %workflow_name,
                workflow_id = %workflow_id,
                "Starting detached workflow execution"
            );

            match run_workflow_by_id(workflow_id.clone(), None).await {
                Ok(result) => {
                    tracing::info!(
                        success = result.success,
                        execution_id = %result.execution_id,
                        workflow_name = %result.workflow_name,
                        "Workflow execution completed"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        workflow_name = %workflow_name,
                        workflow_id = %workflow_id,
                        "Truly detached workflow execution failed - error saved to database, UI will poll for updates"
                    );
                }
            }
        });
    }
}
