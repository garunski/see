use crate::components::{Button, ButtonSize, ButtonVariant, PageHeader};
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn PromptFormHeader(
    is_new: bool,
    is_saving: bool,
    is_deleting: bool,
    on_delete_click: EventHandler<()>,
    on_save_click: EventHandler<()>,
) -> Element {
    rsx! {
        PageHeader {
            title: if is_new { "Create prompt".to_string() } else { "Edit prompt".to_string() },
            description: if is_new { "Create a new prompt template".to_string() } else { "Edit prompt template".to_string() },
            actions: Some(rsx! {
                div { class: "flex items-center gap-3",
                    Link {
                        to: Route::UserPromptsListPage {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700",
                        Icon {
                            name: "arrow_left".to_string(),
                            class: Some("-ml-0.5 h-4 w-4".to_string()),
                            size: None,
                            variant: Some("outline".to_string()),
                        }
                        "Back"
                    }
                    if !is_new {
                        Button {
                            variant: ButtonVariant::Danger,
                            size: ButtonSize::Medium,
                            disabled: Some(is_deleting),
                            loading: Some(is_deleting),
                            onclick: move |_| on_delete_click.call(()),
                            "Delete"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Medium,
                        disabled: Some(is_saving),
                        loading: Some(is_saving),
                        onclick: move |_| on_save_click.call(()),
                        if is_saving { "Saving..." } else { "Save" }
                    }
                }
            }),
        }
    }
}
