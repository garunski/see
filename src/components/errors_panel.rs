use dioxus::prelude::*;

#[component]
pub fn ErrorsPanel(errors: Vec<String>) -> Element {
    rsx! {
        if !errors.is_empty() {
            div {
                class: "bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-800/30 rounded-2xl p-6 animate-fade-in",
                div {
                    class: "flex items-center space-x-3 mb-4",
                    div {
                        class: "w-8 h-8 bg-red-500/20 rounded-lg flex items-center justify-center",
                        "⚠️"
                    }
                    h3 {
                        class: "text-lg font-semibold text-red-600 dark:text-red-400",
                        "Errors"
                    }
                }
                div {
                    class: "space-y-3",
                    for error in &errors {
                        div {
                            class: "text-red-700 dark:text-red-300 font-mono text-sm bg-red-100 dark:bg-red-900/30 p-3 rounded-lg border border-red-200 dark:border-red-800/30",
                            {error.clone()}
                        }
                    }
                }
            }
        }
    }
}
