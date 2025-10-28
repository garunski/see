use crate::queries::ExecuteWorkflowMutation;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, Mutation, UseMutation};

pub struct WorkflowMutations {
    pub execute_mutation: UseMutation<ExecuteWorkflowMutation>,
    pub is_executing: Memo<bool>,
}

pub fn use_workflow_mutations() -> WorkflowMutations {
    tracing::debug!("[use_workflow_mutations] Creating mutation");
    let execute_mutation = use_mutation(Mutation::new(ExecuteWorkflowMutation));
    tracing::debug!("[use_workflow_mutations] Mutation created");

    let is_executing = use_memo(move || execute_mutation.read().state().is_loading());

    WorkflowMutations {
        execute_mutation,
        is_executing,
    }
}
