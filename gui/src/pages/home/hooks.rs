use crate::services::workflow::run_workflow_by_id;

pub fn use_workflow_execution() -> impl Fn(String, String) + 'static {
    move |workflow_name: String, workflow_id: String| {
        tracing::info!(
            workflow_name = %workflow_name,
            workflow_id = %workflow_id,
            "User initiated workflow execution from home page"
        );

        tokio::spawn(async move {
            tracing::info!(
                workflow_name = %workflow_name,
                workflow_id = %workflow_id,
                "Starting truly detached workflow execution - completely independent of UI lifecycle"
            );

            match run_workflow_by_id(workflow_id.clone(), None).await {
                Ok(result) => {
                    tracing::info!(
                        success = result.success,
                        execution_id = %result.execution_id,
                        workflow_name = %result.workflow_name,
                        "Truly detached workflow execution completed successfully - result saved to database, UI will poll for updates"
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
