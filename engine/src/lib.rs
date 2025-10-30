pub mod engine;
pub mod errors;
pub mod handlers;
pub mod parser;
pub mod types;

#[cfg(test)]
mod tests;

pub use engine::WorkflowEngine;
pub use errors::*;
pub use parser::parse_workflow;
pub use types::*;

pub async fn execute_workflow_from_json(json: &str) -> Result<WorkflowResult, EngineError> {
    let workflow = parse_workflow(json)?;
    let engine = WorkflowEngine::new();
    engine.execute_workflow(workflow).await
}
