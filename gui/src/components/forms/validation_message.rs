use dioxus::prelude::*;

#[component]
pub fn ValidationMessage(message: Signal<String>) -> Element {
    if message().is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div { class: "mt-2 text-sm text-red-600 dark:text-red-400",
            {message()}
        }
    }
}
