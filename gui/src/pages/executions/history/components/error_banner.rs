use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::icons::Icon;
use dioxus::prelude::*;

#[component]
pub fn ErrorBanner(error: String, on_retry: EventHandler<()>) -> Element {
    rsx! {
        div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    Icon {
                        name: "exclamation_circle".to_string(),
                        class: Some("w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0".to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                    div {
                        h3 { class: "text-sm font-medium text-red-800 dark:text-red-200", "Failed to load history" }
                        p { class: "text-sm text-red-700 dark:text-red-300 mt-1", "{error}" }
                    }
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Small,
                    onclick: move |_| on_retry.call(()),
                    class: "px-3 py-1 text-sm font-medium text-red-800 dark:text-red-200 bg-red-100 dark:bg-red-900/30 hover:bg-red-200 dark:hover:bg-red-900/50".to_string(),
                    "Retry"
                }
            }
        }
    }
}
