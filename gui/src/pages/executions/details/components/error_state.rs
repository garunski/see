use crate::icons::Icon;
use dioxus::prelude::*;

#[component]
pub fn ErrorState(error: String) -> Element {
    rsx! {
        div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6",
            div { class: "flex items-center gap-3",
                Icon {
                    name: "exclamation_circle".to_string(),
                    class: Some("w-6 h-6 text-red-600 dark:text-red-400".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
                div {
                    h3 { class: "text-base font-semibold text-red-800 dark:text-red-200", "Error Loading Workflow" }
                    p { class: "text-red-700 dark:text-red-300 mt-1", "{error}" }
                }
            }
        }
    }
}
