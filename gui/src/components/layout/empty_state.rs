use dioxus::prelude::*;

#[component]
pub fn EmptyState(message: String) -> Element {
    rsx! {
        div { class: "px-6 py-12 text-center",
            div { class: "text-zinc-500 dark:text-zinc-400",
                {message}
            }
        }
    }
}
