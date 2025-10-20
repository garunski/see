use dioxus::prelude::*;
use see_core::Theme;

#[component]
pub fn SettingsScreen(
    settings: Signal<crate::state::SettingsState>,
    on_theme_change: EventHandler<Theme>,
    on_close: EventHandler<()>,
) -> Element {
    let current_theme = settings.read().settings.theme;

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| on_close.call(()),

            div {
                class: "bg-white dark:bg-zinc-900 rounded-lg shadow-xl p-6 max-w-md w-full mx-4",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "flex items-center justify-between mb-6",
                    h2 { class: "text-2xl font-bold text-zinc-950 dark:text-white", "Settings" }
                    button {
                        class: "text-zinc-500 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                div { class: "space-y-6",
                    // Theme Section
                    div {
                        h3 { class: "text-lg font-semibold text-zinc-950 dark:text-white mb-3", "Appearance" }
                        div { class: "space-y-2",
                            for theme in [Theme::Light, Theme::Dark, Theme::System] {
                                button {
                                    class: format!(
                                        "w-full flex items-center justify-between px-4 py-3 rounded-lg border-2 transition-colors {}",
                                        if current_theme == theme {
                                            "border-blue-500 bg-blue-50 dark:bg-blue-900"
                                        } else {
                                            "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
                                        }
                                    ),
                                    onclick: move |_| on_theme_change.call(theme),

                                    div { class: "flex items-center gap-3",
                                        div { class: "text-2xl",
                                            match theme {
                                                Theme::Light => "☀️",
                                                Theme::Dark => "🌙",
                                                Theme::System => "💻",
                                            }
                                        }
                                        div { class: "text-left",
                                            div { class: "font-medium text-zinc-950 dark:text-white",
                                                match theme {
                                                    Theme::Light => "Light",
                                                    Theme::Dark => "Dark",
                                                    Theme::System => "System",
                                                }
                                            }
                                            div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                                                match theme {
                                                    Theme::Light => "Always use light theme",
                                                    Theme::Dark => "Always use dark theme",
                                                    Theme::System => "Match system preference",
                                                }
                                            }
                                        }
                                    }

                                    if current_theme == theme {
                                        div { class: "text-blue-500", "✓" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
