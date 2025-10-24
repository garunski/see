use crate::icons::Icon;
use dioxus::prelude::*;

#[component]
pub fn ErrorList(errors: Vec<String>) -> Element {
    if errors.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            h3 { class: "text-base font-semibold text-red-800 dark:text-red-200 mb-4", "Errors" }
            div { class: "space-y-3",
                for error in errors.iter() {
                    div { class: "p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg",
                        div { class: "flex items-start gap-3",
                            Icon {
                                name: "exclamation_circle".to_string(),
                                class: Some("w-5 h-5 text-red-600 dark:text-red-400 mt-0.5 flex-shrink-0".to_string()),
                                size: None,
                                variant: Some("outline".to_string()),
                            }
                            div { class: "text-sm text-red-800 dark:text-red-200", "{error}" }
                        }
                    }
                }
            }
        }
    }
}
