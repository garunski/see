use dioxus::prelude::*;

#[component]
pub fn PageHeader(title: String, description: String, actions: Option<Element>) -> Element {
    rsx! {
        div { class: "flex items-center justify-between",
            div {
                h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "{title}" }
                p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "{description}" }
            }
            if let Some(actions) = actions {
                {actions}
            }
        }
    }
}
