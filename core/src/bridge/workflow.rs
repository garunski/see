use crate::errors::CoreError;
use crate::validation;
use s_e_e_engine::{EngineWorkflow, WorkflowResult as EngineWorkflowResult};
use s_e_e_persistence::WorkflowDefinition;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub execution_id: String,
    pub tasks: Vec<s_e_e_engine::TaskInfo>,
    pub audit_trail: Vec<s_e_e_engine::AuditEntry>,
    pub per_task_logs: std::collections::HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}

pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;

pub fn workflow_definition_to_engine(
    workflow: &WorkflowDefinition,
) -> Result<EngineWorkflow, CoreError> {
    validation::validate_workflow_json(&workflow.content).map_err(CoreError::Validation)?;

    let parsed = s_e_e_engine::parse_workflow(&workflow.content)
        .map_err(|e| CoreError::Engine(s_e_e_engine::EngineError::Parser(e)))?;

    Ok(parsed)
}

pub fn engine_result_to_core_result(
    result: EngineWorkflowResult,
    execution_id: String,
) -> WorkflowResult {
    WorkflowResult {
        success: result.success,
        workflow_name: result.workflow_name,
        execution_id,
        tasks: result.tasks,
        audit_trail: result.audit_trail,
        per_task_logs: result.per_task_logs,
        errors: result.errors,
    }
}
