use dioxus::prelude::*;

#[component]
pub fn SectionCard(title: Option<String>, children: Element, padding: Option<String>) -> Element {
    let padding_class = padding.unwrap_or_else(|| "p-8".to_string());

    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm",
            if let Some(title) = title {
                div { class: "px-6 py-4 border-b border-zinc-200 dark:border-zinc-700",
                    h3 { class: "text-base font-semibold text-zinc-900 dark:text-white", "{title}" }
                }
            }
            div { class: padding_class,
                {children}
            }
        }
    }
}
