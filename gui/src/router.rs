use crate::components::Sidebar;
use crate::pages::settings::components::{
    WorkflowEditPage, WorkflowEditPageNew, WorkflowsListPage,
};
use crate::pages::workflow::{UploadPage, WorkflowDetailsPage};
use crate::pages::{HistoryPage, HomePage, SettingsPage};
use dioxus::prelude::*;
use dioxus_router::prelude::{Link, Outlet, Routable};

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        HomePage {},
        #[route("/workflows/upload")]
        UploadPage {},
        #[route("/history")]
        HistoryPage {},
        #[route("/history/:id")]
        WorkflowDetailsPage { id: String },
        #[route("/settings")]
        SettingsPage {},
        #[route("/settings/workflows")]
        WorkflowsListPage {},
        #[route("/settings/workflows/new")]
        WorkflowEditPageNew {},
        #[route("/settings/workflows/edit/:id")]
        WorkflowEditPage { id: String },
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
pub fn AppLayout() -> Element {
    let mut show_sidebar = use_signal(|| false);

    rsx! {
        div { class: "relative isolate flex min-h-svh w-full bg-white max-sm:flex-col sm:bg-zinc-100 dark:bg-zinc-900 dark:sm:bg-zinc-950",
            div { class: "fixed inset-y-0 left-0 w-48 max-sm:hidden",
                Sidebar {}
            }

            header { class: "flex items-center px-4 sm:hidden",
                div { class: "py-2.5",
                    button {
                        class: "inline-flex items-center justify-center rounded-lg p-2 text-zinc-950 hover:bg-zinc-950/5 dark:text-white dark:hover:bg-white/5",
                        onclick: move |_| show_sidebar.set(true),
                        "aria-label": "Open navigation",
                        // OpenMenuIcon SVG
                        svg {
                            class: "h-6 w-6",
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            path { d: "M2 6.75C2 6.33579 2.33579 6 2.75 6H17.25C17.6642 6 18 6.33579 18 6.75C18 7.16421 17.6642 7.5 17.25 7.5H2.75C2.33579 7.5 2 7.16421 2 6.75ZM2 13.25C2 12.8358 2.33579 12.5 2.75 12.5H17.25C17.6642 12.5 18 12.8358 18 13.25C18 13.6642 17.6642 14 17.25 14H2.75C2.33579 14 2 13.6642 2 13.25Z" }
                        }
                    }
                }
                div { class: "min-w-0 flex-1" }
            }

            if *show_sidebar.read() {
                div { class: "sm:hidden",
                    // Backdrop
                    div {
                        class: "fixed inset-0 bg-black/30 transition data-closed:opacity-0 data-enter:duration-300 data-enter:ease-out data-leave:duration-200 data-leave:ease-in",
                        onclick: move |_| show_sidebar.set(false)
                    }
                    // Dialog panel
                    div { class: "fixed inset-y-0 w-full max-w-80 p-2 transition duration-300 ease-in-out",
                        div { class: "flex h-full flex-col rounded-lg bg-white shadow-xs ring-1 ring-zinc-950/5 dark:bg-zinc-900 dark:ring-white/10",
                            div { class: "-mb-3 px-4 pt-3",
                                button {
                                    class: "inline-flex items-center justify-center rounded-lg p-2 text-zinc-950 hover:bg-zinc-950/5 dark:text-white dark:hover:bg-white/5",
                                    onclick: move |_| show_sidebar.set(false),
                                    "aria-label": "Close navigation",
                                    // CloseMenuIcon SVG
                                    svg {
                                        class: "h-6 w-6",
                                        view_box: "0 0 20 20",
                                        fill: "currentColor",
                                        path { d: "M6.28 5.22a.75.75 0 0 0-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 1 0 1.06 1.06L10 11.06l3.72 3.72a.75.75 0 1 0 1.06-1.06L11.06 10l3.72-3.72a.75.75 0 0 0-1.06-1.06L10 8.94 6.28 5.22Z" }
                                    }
                                }
                            }
                            Sidebar {}
                        }
                    }
                }
            }

            main { class: "flex flex-1 flex-col pb-2 sm:min-w-0 sm:pt-2 sm:pr-2 sm:pl-48",
                div { class: "grow p-6 sm:rounded-lg sm:bg-white sm:p-10 sm:shadow-xs sm:ring-1 sm:ring-zinc-950/5 dark:sm:bg-zinc-900 dark:sm:ring-white/10",
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
