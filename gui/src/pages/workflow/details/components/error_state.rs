use dioxus::prelude::*;

#[component]
pub fn ErrorState(error: String) -> Element {
    rsx! {
        div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6",
            div { class: "flex items-center gap-3",
                svg {
                    class: "w-6 h-6 text-red-600 dark:text-red-400",
                    view_box: "0 0 20 20",
                    fill: "currentColor",
                    path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                }
                div {
                    h3 { class: "text-base font-semibold text-red-800 dark:text-red-200", "Error Loading Workflow" }
                    p { class: "text-red-700 dark:text-red-300 mt-1", "{error}" }
                }
            }
        }
    }
}
