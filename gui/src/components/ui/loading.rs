use crate::components::ui::{Icon, IconName};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct LoadingProps {
    pub class: Option<String>,
}

#[component]
pub fn Loading(props: LoadingProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center py-16",
            Icon {
                name: IconName::Spinner,
                class: props.class
            }
        }
    }
}
