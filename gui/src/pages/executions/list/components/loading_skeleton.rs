use dioxus::prelude::*;

#[component]
pub fn LoadingSkeleton() -> Element {
    rsx! {
        div { class: "space-y-4",
            for _ in 0..3 {
                div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-6 animate-pulse",
                    div { class: "flex items-center justify-between",
                        div { class: "flex-1 min-w-0",
                            div { class: "flex items-center gap-4 mb-3",
                                div { class: "h-4 bg-zinc-200 dark:bg-zinc-700 rounded w-48" }
                                div { class: "h-6 bg-zinc-200 dark:bg-zinc-700 rounded-full w-16" }
                            }
                            div { class: "h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-32 mb-2" }
                            div { class: "h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-24" }
                        }
                        div { class: "h-8 w-8 bg-zinc-200 dark:bg-zinc-700 rounded" }
                    }
                }
            }
        }
    }
}
