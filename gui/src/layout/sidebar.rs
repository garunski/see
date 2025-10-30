use super::router::Route;
use crate::icons::Icon;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "fixed inset-y-0 left-0 w-48 bg-zinc-100 dark:bg-zinc-950",
            nav { class: "flex h-full min-h-0 flex-col",
                div { class: "flex flex-col border-b border-zinc-950/5 p-4 dark:border-white/5",
                    div { class: "flex items-center gap-3",
                        Icon {
                            name: "logo".to_string(),
                            class: Some("shrink-0".to_string()),
                            size: Some("w-16 h-16".to_string()),
                            variant: None,
                        }
                        span { class: "text-sm font-semibold text-zinc-900 dark:text-white", "S-E-E" }
                    }
                }

                div { class: "flex flex-1 flex-col overflow-y-auto p-4",

                    div { class: "flex flex-col gap-0.5",
                        Link {
                            to: Route::HomePage {},
                            class: "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-sm font-medium text-zinc-900 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                            Icon {
                                name: "home".to_string(),
                                class: Some("shrink-0 fill-zinc-500 data-hover:fill-zinc-900 data-active:fill-zinc-900 dark:fill-zinc-400 dark:data-hover:fill-white dark:data-active:fill-white".to_string()),
                                size: Some("w-4 h-4".to_string()),
                            }
                            span { class: "truncate", "Home" }
                        }
                        Link {
                            to: Route::ExecutionListPage {},
                            class: "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-sm font-medium text-zinc-900 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                            Icon {
                                name: "executions".to_string(),
                                class: Some("shrink-0 fill-zinc-500 data-hover:fill-zinc-900 data-active:fill-zinc-900 dark:fill-zinc-400 dark:data-hover:fill-white dark:data-active:fill-white".to_string()),
                                size: Some("w-4 h-4".to_string()),
                            }
                            span { class: "truncate", "Executions" }
                        }
                    }


                    div { class: "mt-4",
                        div { class: "px-2 text-xs font-semibold text-zinc-500 dark:text-zinc-400 mb-1",
                            "Manage"
                        }
                        div { class: "flex flex-col gap-0.5",
                            Link {
                                to: Route::WorkflowsListPage {},
                                class: "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-sm font-medium text-zinc-900 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                                Icon {
                                    name: "workflows".to_string(),
                                    class: Some("shrink-0 fill-zinc-500 data-hover:fill-zinc-900 data-active:fill-zinc-900 dark:fill-zinc-400 dark:data-hover:fill-white dark:data-active:fill-white".to_string()),
                                    size: Some("w-4 h-4".to_string()),
                                }
                                span { class: "truncate", "Workflows" }
                            }
                            Link {
                                to: Route::UserPromptsListPage {},
                                class: "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-sm font-medium text-zinc-900 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                                Icon {
                                    name: "prompts".to_string(),
                                    class: Some("shrink-0 fill-zinc-500 data-hover:fill-zinc-900 data-active:fill-zinc-900 dark:fill-zinc-400 dark:data-hover:fill-white dark:data-active:fill-white".to_string()),
                                    size: Some("w-4 h-4".to_string()),
                                }
                                span { class: "truncate", "Prompts" }
                            }
                            Link {
                                to: Route::SettingsPage {},
                                class: "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-sm font-medium text-zinc-900 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                                Icon {
                                    name: "settings".to_string(),
                                    class: Some("shrink-0 fill-zinc-500 data-hover:fill-zinc-900 data-active:fill-zinc-900 dark:fill-zinc-400 dark:data-hover:fill-white dark:data-active:fill-white".to_string()),
                                    size: Some("w-4 h-4".to_string()),
                                }
                                span { class: "truncate", "Settings" }
                            }
                        }
                    }
                }
            }
        }
    }
}
