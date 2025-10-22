use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct IconProps {
    pub name: IconName,
    pub class: Option<String>,
}

#[derive(PartialEq, Clone)]
pub enum IconName {
    BackArrow,
    Error,
    #[allow(dead_code)]
    Checkmark,
    #[allow(dead_code)]
    X,
    Spinner,
}

impl IconName {
    fn svg_path(&self) -> &'static str {
        match self {
            IconName::BackArrow => "M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z",
            IconName::Error => "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z",
            IconName::Checkmark => "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z",
            IconName::X => "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z",
            IconName::Spinner => "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
        }
    }

    fn default_class(&self) -> &'static str {
        match self {
            IconName::Spinner => "animate-spin w-8 h-8 border-2 border-zinc-300 border-t-zinc-900 rounded-full dark:border-zinc-600 dark:border-t-zinc-100",
            _ => "w-5 h-5",
        }
    }
}

#[component]
pub fn Icon(props: IconProps) -> Element {
    let class = props
        .class
        .unwrap_or_else(|| props.name.default_class().to_string());

    rsx! {
        svg {
            class: class,
            view_box: "0 0 20 20",
            fill: "currentColor",
            path { d: props.name.svg_path() }
        }
    }
}
