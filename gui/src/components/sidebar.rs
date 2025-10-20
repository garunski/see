use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "fixed inset-y-0 left-0 w-64 bg-white dark:bg-zinc-900 border-r border-zinc-200 dark:border-zinc-800",
            nav { class: "flex h-full min-h-0 flex-col",
                // Header
                div { class: "flex flex-col border-b border-zinc-200 dark:border-zinc-800 p-4",
                    div { class: "flex items-center gap-3",
                        div { class: "w-8 h-8 bg-zinc-900 dark:bg-white rounded-lg flex items-center justify-center text-white dark:text-zinc-900 text-lg font-semibold", "⚡" }
                        span { class: "text-lg font-semibold text-zinc-950 dark:text-white", "S-E-E" }
                    }
                }

                // Navigation
                div { class: "flex flex-1 flex-col overflow-y-auto p-4",
                    div { class: "flex flex-col gap-0.5",
                        Link {
                            to: Route::UploadPage {},
                            class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 sm:py-2 sm:text-sm/5 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                            // HomeIcon SVG
                            svg {
                                class: "w-5 h-5",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path { d: "M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z" }
                            }
                            span { "Upload" }
                        }
                        Link {
                            to: Route::HistoryPage {},
                            class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 sm:py-2 sm:text-sm/5 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                            // Square2StackIcon SVG
                            svg {
                                class: "w-5 h-5",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path { d: "M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" }
                            }
                            span { "History" }
                        }
                        Link {
                            to: Route::SettingsPage {},
                            class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 sm:py-2 sm:text-sm/5 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                            // Cog6ToothIcon SVG
                            svg {
                                class: "w-5 h-5",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zM10 6a4 4 0 100 8 4 4 0 000-8z" }
                            }
                            span { "Settings" }
                        }
                    }
                }
            }
        }
    }
}
