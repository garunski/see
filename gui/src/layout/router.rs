use crate::components::{IconButton, IconButtonSize, IconButtonVariant};
use crate::pages::{
    HistoryPage, HomePage, SettingsPage, UploadPage, UserPromptEditPage, UserPromptEditPageNew,
    UserPromptsListPage, WorkflowDetailsPage, WorkflowEditPage, WorkflowEditPageNew,
    WorkflowVisualizerPage, WorkflowsListPage,
};
use dioxus::prelude::*;
use dioxus_router::prelude::{Link, Outlet, Routable};

use super::sidebar::Sidebar;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        HomePage {},
        #[route("/workflows/upload")]
        UploadPage {},
        #[route("/workflows/visualize/:id")]
        WorkflowVisualizerPage { id: String },
        #[route("/workflows")]
        WorkflowsListPage {},
        #[route("/workflows/new")]
        WorkflowEditPageNew {},
        #[route("/workflows/edit/:id")]
        WorkflowEditPage { id: String },
        #[route("/executions/history")]
        HistoryPage {},
        #[route("/executions/details/:id")]
        WorkflowDetailsPage { id: String },
        #[route("/prompts")]
        UserPromptsListPage {},
        #[route("/prompts/new")]
        UserPromptEditPageNew {},
        #[route("/prompts/edit/:id")]
        UserPromptEditPage { id: String },
        #[route("/settings")]
        SettingsPage {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
pub fn AppLayout() -> Element {
    let mut show_sidebar = use_signal(|| false);

    rsx! {
        div { class: "relative isolate flex h-screen w-full bg-white max-sm:flex-col sm:bg-zinc-100 dark:bg-zinc-900 dark:sm:bg-zinc-950",
            div { class: "fixed inset-y-0 left-0 w-48 max-sm:hidden",
                Sidebar {}
            }

            header { class: "flex items-center px-4 sm:hidden",
                div { class: "py-2.5",
                    IconButton {
                        variant: IconButtonVariant::Ghost,
                        size: IconButtonSize::Medium,
                        onclick: move |_| show_sidebar.set(true),
                        class: Some("p-2".to_string()),
                        icon: Some("bars_3".to_string()),
                        icon_variant: "outline".to_string(),
                        ""
                    }
                }
                div { class: "min-w-0 flex-1" }
            }

            if *show_sidebar.read() {
                div { class: "sm:hidden",
                    div {
                        class: "fixed inset-0 bg-black/30 transition data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in",
                        onclick: move |_| show_sidebar.set(false)
                    }
                    div { class: "fixed inset-y-0 w-full max-w-80 p-2 transition duration-300 ease-in-out",
                        div { class: "flex h-full flex-col rounded-lg bg-white shadow-xs ring-1 ring-zinc-950/5 dark:bg-zinc-900 dark:ring-white/10",
                            div { class: "-mb-3 px-4 pt-3",
                                IconButton {
                                    variant: IconButtonVariant::Ghost,
                                    size: IconButtonSize::Medium,
                                    onclick: move |_| show_sidebar.set(false),
                                    class: Some("p-2".to_string()),
                                    icon: Some("x".to_string()),
                                    icon_variant: "outline".to_string(),
                                    ""
                                }
                            }
                            Sidebar {}
                        }
                    }
                }
            }

            main { class: "flex flex-1 flex-col pb-2 sm:min-w-0 sm:pt-2 sm:pr-2 sm:pl-48 min-h-0",
                div { class: "flex-1 p-6 sm:rounded-lg sm:bg-white sm:p-10 sm:shadow-xs sm:ring-1 sm:ring-zinc-950/5 dark:sm:bg-zinc-900 dark:sm:ring-white/10 overflow-y-auto",
                    div { class: "mx-auto max-w-6xl",
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-center",
                h1 { class: "text-2xl font-bold text-zinc-900 dark:text-white mb-4", "Page not found" }
                p { class: "text-zinc-600 dark:text-zinc-400 mb-2", "The page you're looking for doesn't exist." }
                Link { to: Route::HomePage {}, class: "text-blue-600 dark:text-blue-400 hover:underline", "Go Home" }
            }
        }
    }
}
