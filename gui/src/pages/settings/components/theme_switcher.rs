use crate::icons::Icon;
use dioxus::prelude::*;
use s_e_e_core::Theme;

#[derive(Debug, PartialEq, Clone, Props)]
pub struct ThemeSwitcherProps {
    pub current_theme: Theme,
    pub on_theme_change: EventHandler<Theme>,
}

#[component]
pub fn ThemeSwitcher(props: ThemeSwitcherProps) -> Element {
    let current_theme = props.current_theme.clone();
    let on_theme_change = props.on_theme_change;

    rsx! {
        div { class: "inline-flex rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1",
            button {
                class: format!("flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors cursor-pointer {}",
                    match current_theme == Theme::System {
                        true => "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-white shadow-sm",
                        false => "text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white"
                    }
                ),
                onclick: move |_| {
                    tracing::info!("[ThemeSwitcher] User clicked System");
                    on_theme_change.call(Theme::System);
                },
                Icon { name: "computer-desktop".to_string(), size: Some("w-4 h-4".to_string()) }
                "System"
            }
            button {
                class: format!("flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors cursor-pointer {}",
                    match current_theme == Theme::Light {
                        true => "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-white shadow-sm",
                        false => "text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white"
                    }
                ),
                onclick: move |_| {
                    tracing::info!("[ThemeSwitcher] User clicked Light");
                    on_theme_change.call(Theme::Light);
                },
                Icon { name: "sun".to_string(), size: Some("w-4 h-4".to_string()) }
                "Light"
            }
            button {
                class: format!("flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors cursor-pointer {}",
                    match current_theme == Theme::Dark {
                        true => "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-white shadow-sm",
                        false => "text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white"
                    }
                ),
                onclick: move |_| {
                    tracing::info!("[ThemeSwitcher] User clicked Dark");
                    on_theme_change.call(Theme::Dark);
                },
                Icon { name: "moon".to_string(), size: Some("w-4 h-4".to_string()) }
                "Dark"
            }
        }
    }
}
