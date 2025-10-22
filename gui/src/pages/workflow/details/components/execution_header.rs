use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn ExecutionHeader() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "flex items-center gap-4",
            button {
                onclick: move |_| navigator.go_back(),
                class: "flex items-center gap-2 px-3 py-2 text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white transition-colors rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800",
                svg {
                    class: "w-5 h-5",
                    view_box: "0 0 20 20",
                    fill: "currentColor",
                    path { d: "M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" }
                }
                span { "Back to History" }
            }
            h1 { class: "text-lg font-semibold text-zinc-950 dark:text-white", "Workflow Execution Details" }
        }
    }
}
