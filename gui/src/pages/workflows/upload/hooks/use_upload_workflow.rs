use crate::queries::CreateWorkflowMutation;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, Mutation, UseMutation};
use s_e_e_core::WorkflowDefinition;

pub struct UploadWorkflowState {
    pub create_mutation: UseMutation<CreateWorkflowMutation>,
    pub is_saving: Memo<bool>,
}

pub fn use_upload_workflow() -> UploadWorkflowState {
    let create_mutation = use_mutation(Mutation::new(CreateWorkflowMutation));

    let is_saving = use_memo(move || create_mutation.read().state().is_loading());

    UploadWorkflowState {
        create_mutation,
        is_saving,
    }
}

pub fn upload_workflow_from_json(
    mutation: &UseMutation<CreateWorkflowMutation>,
    workflow: WorkflowDefinition,
) -> Result<(), String> {
    let json_str = serde_json::to_string(&workflow)
        .map_err(|e| format!("Failed to serialize workflow: {}", e))?;

    mutation.mutate(json_str);
    Ok(())
}
