use crate::components::Sidebar;
use crate::pages::workflow::{UploadPage, WorkflowDetailsPage};
use crate::pages::{HistoryPage, SettingsPage};
use dioxus::prelude::*;
use dioxus_router::prelude::{Link, Outlet, Routable};

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        UploadPage {},
        #[route("/history")]
        HistoryPage {},
        #[route("/history/:id")]
        WorkflowDetailsPage { id: String },
        #[route("/settings")]
        SettingsPage {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        div { class: "flex min-h-screen bg-zinc-100 dark:bg-zinc-950",
            Sidebar {}
            main { class: "flex-1 ml-64",
                div { class: "p-6",
                    div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-8",
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
                Link { to: Route::UploadPage {}, class: "text-blue-600 dark:text-blue-400 hover:underline", "Go Home" }
            }
        }
    }
}
