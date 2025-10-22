use see_core::{
    errors::CoreError, execute_workflow, execute_workflow_from_content, OutputCallback,
    WorkflowResult,
};

pub async fn run_workflow_from_content(
    content: String,
    output: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow_from_content(&content, output).await
}

pub async fn run_workflow(
    file_path: String,
    output: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError> {
    execute_workflow(&file_path, output).await
}
