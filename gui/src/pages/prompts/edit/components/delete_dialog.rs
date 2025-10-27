use crate::components::ConfirmDialog;
use dioxus::prelude::*;

#[component]
pub fn PromptDeleteDialog(
    show: bool,
    prompt_id: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        ConfirmDialog {
            show,
            title: "Delete Prompt?".to_string(),
            message: format!("Are you sure you want to delete the prompt '{}'? This action cannot be undone.", prompt_id),
            confirm_text: "Delete".to_string(),
            cancel_text: "Cancel".to_string(),
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
        }
    }
}
