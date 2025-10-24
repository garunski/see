use crate::hooks::use_workflows;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

use super::components::{ActionCard, ActionIcon, WorkflowList};

#[component]
pub fn HomePage() -> Element {
    let workflows = use_workflows();

    rsx! {
        div { class: "space-y-8",
            div {
                h1 { class: "text-2xl font-bold text-zinc-900 dark:text-white", "Welcome to See" }
                p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "Your workflow automation platform" }
            }

            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                ActionCard {
                    title: "Create Workflow".to_string(),
                    description: "Design a new workflow".to_string(),
                    icon: ActionIcon::Plus,
                    route: Route::WorkflowEditPageNew {},
                }

                ActionCard {
                    title: "Upload & Execute".to_string(),
                    description: "Run a workflow file".to_string(),
                    icon: ActionIcon::Upload,
                    route: Route::UploadPage {},
                }

                ActionCard {
                    title: "View History".to_string(),
                    description: "Check execution logs".to_string(),
                    icon: ActionIcon::History,
                    route: Route::HistoryPage {},
                }
            }

            div { class: "space-y-4",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Your Workflows" }
                    Link {
                        to: Route::WorkflowsListPage {},
                        class: "text-sm text-blue-600 dark:text-blue-400 hover:text-blue-900 dark:hover:text-blue-300",
                        "View all"
                    }
                }

                WorkflowList { workflows: workflows() }
            }
        }
    }
}
