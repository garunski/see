use dioxus::prelude::*;

#[component]
pub fn Toast(message: Option<String>, on_dismiss: EventHandler<()>) -> Element {
    if let Some(ref toast) = message {
        rsx! {
            div {
                class: "fixed top-6 right-6 z-50 animate-slide-in",
                div {
                    class: "bg-white dark:bg-zinc-900 shadow-xl ring-1 ring-zinc-950/5 dark:ring-white/10 rounded-2xl p-4 max-w-sm",
                    div {
                        class: "flex items-center justify-between",
                        span {
                            class: "text-zinc-950 dark:text-white font-medium",
                            {toast.clone()}
                        }
                        button {
                            class: "ml-4 text-zinc-500 dark:text-zinc-400 hover:text-zinc-950 dark:hover:text-white transition-colors",
                            onclick: move |_| on_dismiss.call(()),
                            "✕"
                        }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
