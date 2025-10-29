use crate::queries::use_workflows_query;
use s_e_e_core::WorkflowDefinition;

pub fn use_workflows_list() -> Result<Vec<WorkflowDefinition>, String> {
    let (state, _refetch) = use_workflows_query();

    if state.is_loading {
        Err("Loading workflows...".to_string())
    } else if state.is_error {
        Err(state
            .error
            .clone()
            .unwrap_or_else(|| "Failed to load workflows".to_string()))
    } else {
        Ok(state.data.clone().unwrap_or_default())
    }
}
