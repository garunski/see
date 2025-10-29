use crate::components::ConfirmDialog;
use dioxus::prelude::*;

#[component]
pub fn ExecutionDeleteDialog(
    show: bool,
    execution_id: String,
    workflow_name: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        ConfirmDialog {
            show,
            title: "Delete Execution?".to_string(),
            message: format!("Are you sure you want to delete the execution '{}'? This will permanently delete the execution, all its tasks, and associated data. This action cannot be undone.", workflow_name),
            confirm_text: "Delete".to_string(),
            cancel_text: "Cancel".to_string(),
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
        }
    }
}
