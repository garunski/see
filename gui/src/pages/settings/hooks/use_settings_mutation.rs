use crate::queries::use_update_settings_mutation;
use dioxus::prelude::*;
use s_e_e_core::AppSettings;
use s_e_e_dioxus_query::prelude::MutationState;

pub struct SettingsMutation {
    #[allow(dead_code)]
    pub state: Signal<MutationState<()>>,
    pub mutate_fn: std::rc::Rc<dyn Fn(AppSettings)>,
}

pub fn use_settings_mutation() -> SettingsMutation {
    let (state, mutate_fn) = use_update_settings_mutation();

    SettingsMutation {
        state,
        mutate_fn: std::rc::Rc::new(mutate_fn),
    }
}
