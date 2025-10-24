use dioxus::prelude::*;

#[component]
pub fn LoadingState() -> Element {
    rsx! {
        div { class: "flex items-center justify-center py-16",
            div { class: "animate-spin w-8 h-8 border-2 border-zinc-300 border-t-zinc-900 rounded-full dark:border-zinc-600 dark:border-t-zinc-100" }
        }
    }
}
