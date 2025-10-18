use dioxus::prelude::*;

#[component]
pub fn OutputLogsPanel(
    logs: Vec<String>,
    show_logs: bool,
    on_toggle: EventHandler<()>,
    on_copy: EventHandler<String>,
) -> Element {
    if !logs.is_empty() {
        rsx! {
            div {
                class: "bg-white dark:bg-zinc-900 shadow-xs ring-1 ring-zinc-950/5 dark:ring-white/10 rounded-2xl overflow-hidden",
                button {
                    class: "w-full px-6 py-4 text-left flex items-center justify-between data-hover:bg-zinc-950/5 dark:data-hover:bg-white/5 transition-colors duration-200",
                    onclick: move |_| on_toggle.call(()),
                    div {
                        class: "flex items-center space-x-3",
                        div {
                            class: "w-8 h-8 bg-blue-500/20 rounded-lg flex items-center justify-center",
                            "📋"
                        }
                        div {
                            span {
                                class: "font-semibold text-zinc-950 dark:text-white text-lg",
                                "Execution Output"
                            }
                            span {
                                class: "text-sm text-zinc-500 dark:text-zinc-400 ml-2",
                                "({logs.len()} lines)"
                            }
                        }
                    }
                    div {
                        class: "transform transition-transform duration-200",
                        class: if show_logs { "rotate-180" } else { "" },
                        "▼"
                    }
                }
                if show_logs {
                    div {
                        class: "border-t border-zinc-950/5 dark:border-white/5",
                        div {
                            class: "bg-zinc-50 dark:bg-zinc-950 text-zinc-950 dark:text-zinc-100 p-6 font-mono text-sm max-h-80 overflow-y-auto",
                            {logs.join("\n")}
                        }
                        div {
                            class: "px-6 py-4 bg-zinc-50 dark:bg-zinc-800 flex justify-end",
                            button {
                                class: "px-4 py-2 bg-blue-500/20 hover:bg-blue-500/30 text-blue-600 dark:text-blue-400 rounded-xl text-sm font-medium transition-colors duration-200 border border-blue-500/30",
                                onclick: move |_| {
                                    let logs_text = logs.join("\n");
                                    on_copy.call(logs_text);
                                },
                                "Copy Output"
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
