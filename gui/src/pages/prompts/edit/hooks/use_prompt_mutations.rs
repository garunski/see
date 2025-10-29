use crate::queries::prompt_queries::{
    use_create_prompt_mutation, use_delete_prompt_mutation, use_update_prompt_mutation,
};
use dioxus::prelude::*;
use dioxus_query_custom::prelude::MutationState;
use s_e_e_core::Prompt;

pub struct PromptMutations {
    pub create_state: Signal<MutationState<()>>,
    pub create_fn: Box<dyn Fn(Prompt)>,
    pub update_state: Signal<MutationState<()>>,
    pub update_fn: Box<dyn Fn(Prompt)>,
    pub delete_state: Signal<MutationState<()>>,
    pub delete_fn: Box<dyn Fn(String)>,
    pub is_saving: Memo<bool>,
    pub is_deleting: Memo<bool>,
}

pub fn use_prompt_mutations() -> PromptMutations {
    let (create_state, create_fn) = use_create_prompt_mutation();
    let (update_state, update_fn) = use_update_prompt_mutation();
    let (delete_state, delete_fn) = use_delete_prompt_mutation();

    let is_saving =
        use_memo(move || create_state.read().is_loading || update_state.read().is_loading);
    let is_deleting = use_memo(move || delete_state.read().is_loading);

    PromptMutations {
        create_state,
        create_fn: Box::new(create_fn),
        update_state,
        update_fn: Box::new(update_fn),
        delete_state,
        delete_fn: Box::new(delete_fn),
        is_saving,
        is_deleting,
    }
}
