use dioxus::prelude::*;

#[component]
pub fn ContextPanel(
    context: serde_json::Value,
    show_context: bool,
    on_toggle: EventHandler<()>,
    on_copy: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "bg-white dark:bg-zinc-900 shadow-xs ring-1 ring-zinc-950/5 dark:ring-white/10 rounded-2xl overflow-hidden",
            button { class: "w-full px-6 py-4 text-left flex items-center justify-between data-hover:bg-zinc-950/5 dark:data-hover:bg-white/5 transition-colors duration-200", onclick: move |_| on_toggle.call(()),
                div { class: "flex items-center space-x-3",
                    div { class: "w-8 h-8 bg-purple-500/20 rounded-lg flex items-center justify-center", "📊" }
                    span { class: "font-semibold text-zinc-950 dark:text-white text-lg", "Final Context" }
                }
                div { class: "transform transition-transform duration-200", class: if show_context { "rotate-180" } else { "" }, "▼" }
            }
            if show_context {
                div { class: "border-t border-zinc-950/5 dark:border-white/5",
                    div { class: "bg-zinc-50 dark:bg-zinc-950 p-6",
                        pre { class: "text-sm text-zinc-950 dark:text-zinc-100 whitespace-pre-wrap overflow-x-auto font-mono",
                            {serde_json::to_string_pretty(&context).unwrap_or_else(|_| "{}".to_string())}
                        }
                    }
                    div { class: "px-6 py-4 bg-zinc-50 dark:bg-zinc-800 flex justify-end",
                        button { class: "px-4 py-2 bg-purple-500/20 hover:bg-purple-500/30 text-purple-600 dark:text-purple-400 rounded-xl text-sm font-medium transition-colors duration-200 border border-purple-500/30", onclick: move |_| on_copy.call(()), "Copy Context" }
                    }
                }
            }
        }
    }
}
