use crate::components::{IconButton, IconButtonSize, IconButtonVariant};
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[derive(Props, PartialEq, Clone)]
pub struct EditorHeaderProps {
    pub is_new: bool,
    pub workflow_id: String,
    pub is_saving: Signal<bool>,
    pub has_unsaved_changes: Signal<bool>,
    pub on_save: EventHandler<()>,
}

#[component]
pub fn EditorHeader(props: EditorHeaderProps) -> Element {
    let EditorHeaderProps {
        is_new,
        workflow_id,
        is_saving,
        has_unsaved_changes,
        on_save,
    } = props;

    let navigator = use_navigator();

    rsx! {
        div { class: "flex items-center justify-between",
            div { class: "flex items-center gap-4",
                IconButton {
                    variant: IconButtonVariant::Ghost,
                    size: IconButtonSize::Medium,
                    onclick: move |_| {
                        if has_unsaved_changes() {
                            // For now, just navigate back - in a real app you'd want a proper confirmation dialog
                            // TODO: Implement proper confirmation dialog using Dioxus components
                        }
                        // Navigate back using Dioxus router
                        navigator.go_back();
                    },
                    class: Some("inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700".to_string()),
                    icon: Some("arrow_left".to_string()),
                    icon_variant: "outline".to_string(),
                    "Back"
                }
                div {
                    h1 { class: "text-xl font-bold text-zinc-900 dark:text-white",
                        if is_new { "Create Workflow" } else { "Edit Workflow" }
                    }
                    p { class: "mt-2 text-zinc-600 dark:text-zinc-400",
                        if is_new { "Create a new workflow definition" } else { "Edit workflow definition" }
                    }
                }
            }
            div { class: "flex items-center gap-3",
                // Edit in JSON button
                if !is_new {
                    IconButton {
                        variant: IconButtonVariant::Ghost,
                        size: IconButtonSize::Medium,
                        onclick: move |_| {
                            navigator.push(Route::WorkflowJsonEditPage { id: workflow_id.clone() });
                        },
                        icon: Some("document_text".to_string()),
                        icon_variant: "outline".to_string(),
                        "Edit in JSON"
                    }
                }

                IconButton {
                    variant: IconButtonVariant::Primary,
                    size: IconButtonSize::Medium,
                    disabled: Some(is_saving()),
                    loading: Some(is_saving()),
                    onclick: move |_| on_save.call(()),
                    icon: if is_saving() { None } else { Some("save".to_string()) },
                    icon_variant: "outline".to_string(),
                    if is_saving() { "Saving..." } else { "Save" }
                }
            }
        }
    }
}
