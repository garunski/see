use crate::components::{IconButton, IconButtonSize, IconButtonVariant};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use super::super::EditMode;

#[derive(Props, PartialEq, Clone)]
pub struct EditorHeaderProps {
    pub is_new: bool,
    pub edit_mode: Signal<EditMode>,
    pub can_reset: Signal<bool>,
    pub is_saving: Signal<bool>,
    pub has_unsaved_changes: Signal<bool>,
    pub on_mode_switch_to_visual: EventHandler<()>,
    pub on_mode_switch_to_json: EventHandler<()>,
    pub on_save: EventHandler<()>,
    pub on_reset: EventHandler<()>,
}

#[component]
pub fn EditorHeader(props: EditorHeaderProps) -> Element {
    let EditorHeaderProps {
        is_new,
        edit_mode,
        can_reset,
        is_saving,
        has_unsaved_changes,
        on_mode_switch_to_visual,
        on_mode_switch_to_json,
        on_save,
        on_reset,
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
                // Mode toggle buttons
                div { class: "flex rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1",
                    IconButton {
                        variant: if edit_mode() == EditMode::Visual { IconButtonVariant::Primary } else { IconButtonVariant::Ghost },
                        size: IconButtonSize::Small,
                        onclick: move |_| on_mode_switch_to_visual.call(()),
                        "Visual Editor"
                    }
                    IconButton {
                        variant: if edit_mode() == EditMode::Json { IconButtonVariant::Primary } else { IconButtonVariant::Ghost },
                        size: IconButtonSize::Small,
                        onclick: move |_| on_mode_switch_to_json.call(()),
                        "JSON Editor"
                    }
                }

                if can_reset() {
                    IconButton {
                        variant: IconButtonVariant::Danger,
                        size: IconButtonSize::Medium,
                        onclick: move |_| on_reset.call(()),
                        icon: Some("arrow_left".to_string()),
                        icon_variant: "outline".to_string(),
                        "Reset to Default"
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
