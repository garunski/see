use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn WorkflowsListPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let workflows = use_memo(move || state_provider.settings.read().get_workflows().clone());

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Workflows" }
                    p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "Manage your workflow definitions" }
                }
                Link {
                    to: Route::WorkflowEditPageNew {},
                    class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                    svg { class: "-ml-0.5 h-5 w-5", view_box: "0 0 20 20", fill: "currentColor",
                        path { d: "M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z" }
                    }
                    "Create workflow"
                }
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm",
                div { class: "px-6 py-4 border-b border-zinc-200 dark:border-zinc-700",
                    h3 { class: "text-base font-semibold text-zinc-900 dark:text-white", "All Workflows" }
                }

                if workflows().is_empty() {
                    div { class: "px-6 py-12 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No workflows yet. Create your first workflow to get started."
                        }
                    }
                } else {
                    div { class: "overflow-hidden",
                        table { class: "min-w-full divide-y divide-zinc-200 dark:divide-zinc-700",
                            thead { class: "bg-zinc-50 dark:bg-zinc-700",
                                tr {
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Name" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Type" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Status" }
                                    th { class: "px-6 py-3 text-right text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Actions" }
                                }
                            }
                            tbody { class: "bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700",
                                for workflow in workflows().iter() {
                                    tr { class: "hover:bg-zinc-50 dark:hover:bg-zinc-700",
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            div { class: "text-sm font-medium text-zinc-900 dark:text-white",
                                                {workflow.get_name()}
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            if workflow.is_default {
                                                span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                                                    "Default"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center rounded-md bg-zinc-50 dark:bg-zinc-800 px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 ring-1 ring-inset ring-zinc-500/10",
                                                    "Custom"
                                                }
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            if workflow.is_default && workflow.is_edited {
                                                span { class: "inline-flex items-center rounded-md bg-yellow-50 dark:bg-yellow-900/20 px-2 py-1 text-xs font-medium text-yellow-700 dark:text-yellow-300 ring-1 ring-inset ring-yellow-600/10",
                                                    "Modified"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center rounded-md bg-green-50 dark:bg-green-900/20 px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-600/10",
                                                    "Active"
                                                }
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                                            Link {
                                                to: Route::WorkflowEditPage { id: workflow.id.clone() },
                                                class: "text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300",
                                                "Edit"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
