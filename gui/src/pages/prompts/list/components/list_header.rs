use crate::components::PageHeader;
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn PromptsListHeader() -> Element {
    rsx! {
        PageHeader {
            title: "Prompts".to_string(),
            description: "Manage your prompt templates".to_string(),
            actions: Some(rsx! {
                Link {
                    to: Route::UserPromptEditPageNew {},
                    class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                    Icon {
                        name: "plus".to_string(),
                        class: Some("-ml-0.5 h-5 w-5".to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                    "Create prompt"
                }
            }),
        }
    }
}
