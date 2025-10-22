use dioxus::prelude::*;

#[component]
pub fn ErrorBanner(error: String, on_retry: EventHandler<()>) -> Element {
    rsx! {
        div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    svg {
                        class: "w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                    }
                    div {
                        h3 { class: "text-sm font-medium text-red-800 dark:text-red-200", "Failed to load history" }
                        p { class: "text-sm text-red-700 dark:text-red-300 mt-1", "{error}" }
                    }
                }
                button {
                    class: "px-3 py-1 text-sm font-medium text-red-800 dark:text-red-200 bg-red-100 dark:bg-red-900/30 hover:bg-red-200 dark:hover:bg-red-900/50 rounded-md transition-colors",
                    onclick: move |_| on_retry.call(()),
                    "Retry"
                }
            }
        }
    }
}
