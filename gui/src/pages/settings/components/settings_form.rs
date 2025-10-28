use crate::pages::settings::components::ThemeSwitcher;
use dioxus::prelude::*;
use s_e_e_core::Theme;

#[derive(Debug, PartialEq, Clone, Props)]
pub struct SettingsFormProps {
    pub theme: Signal<Theme>,
    pub on_theme_change: EventHandler<Theme>,
}

#[component]
pub fn SettingsForm(props: SettingsFormProps) -> Element {
    rsx! {
        div { class: "space-y-6",
            div {
                label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-3", "Theme" }
                ThemeSwitcher {
                    current_theme: props.theme.read().clone(),
                    on_theme_change: props.on_theme_change
                }
            }
        }
    }
}
