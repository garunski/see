use crate::queries::UpdateSettingsMutation;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, Mutation, UseMutation};

pub struct SettingsMutation {
    pub update_mutation: UseMutation<UpdateSettingsMutation>,
    #[allow(dead_code)]
    pub is_saving: dioxus::prelude::Memo<bool>,
}

pub fn use_settings_mutation() -> SettingsMutation {
    let update_mutation = use_mutation(Mutation::new(UpdateSettingsMutation));

    let is_saving = use_memo(move || update_mutation.read().state().is_loading());

    SettingsMutation {
        update_mutation,
        is_saving,
    }
}
