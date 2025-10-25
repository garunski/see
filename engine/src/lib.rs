//! Workflow Engine with Next Tasks Dependency System
//!
//! This module provides a clean implementation of a workflow engine that supports:
//! - Recursive `next_tasks` dependency structure
//! - Parallel execution of independent tasks
//! - Sequential execution based on dependencies
//! - Backward compatibility with existing workflows

pub mod engine;
pub mod errors;
pub mod graph;
pub mod handlers;
pub mod parser;
pub mod types;

#[cfg(test)]
mod tests;

pub use engine::WorkflowEngine;
pub use errors::*;
pub use parser::parse_workflow;
pub use types::*;

/// Execute a workflow from JSON string
pub async fn execute_workflow_from_json(
    json: &str,
    // store: Arc<dyn AuditStore>, // Will be added when integrating with existing codebase
) -> Result<WorkflowResult, EngineError> {
    let workflow = parse_workflow(json)?;
    let engine = WorkflowEngine::new();
    engine.execute_workflow(workflow).await
}
