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
                            svg {
                                class: "w-5 h-5 text-red-600 dark:text-red-400 mt-0.5 flex-shrink-0",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                            }
                            div { class: "text-sm text-red-800 dark:text-red-200", "{error}" }
                        }
                    }
                }
            }
        }
    }
}
