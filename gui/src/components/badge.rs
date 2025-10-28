use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BadgeColor {
    Red,
    Amber,
    Emerald,
    Blue,
    Zinc,
}

impl BadgeColor {
    fn as_classes(&self) -> &'static str {
        match self {
            BadgeColor::Red => {
                "bg-red-500/15 text-red-700 group-data-hover:bg-red-500/25 dark:bg-red-500/10 dark:text-red-400 dark:group-data-hover:bg-red-500/20"
            }
            BadgeColor::Amber => {
                "bg-amber-400/20 text-amber-700 group-data-hover:bg-amber-400/30 dark:bg-amber-400/10 dark:text-amber-400 dark:group-data-hover:bg-amber-400/15"
            }
            BadgeColor::Emerald => {
                "bg-emerald-500/15 text-emerald-700 group-data-hover:bg-emerald-500/25 dark:bg-emerald-500/10 dark:text-emerald-400 dark:group-data-hover:bg-emerald-500/20"
            }
            BadgeColor::Blue => {
                "bg-blue-500/15 text-blue-700 group-data-hover:bg-blue-500/25 dark:text-blue-400 dark:group-data-hover:bg-blue-500/25"
            }
            BadgeColor::Zinc => {
                "bg-zinc-600/10 text-zinc-700 group-data-hover:bg-zinc-600/20 dark:bg-white/5 dark:text-zinc-400 dark:group-data-hover:bg-white/10"
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct BadgeProps {
    #[props(default = BadgeColor::Zinc)]
    pub color: BadgeColor,
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let BadgeProps {
        color,
        class,
        children,
    } = props;

    let badge_classes = format!(
        "inline-flex items-center gap-x-1.5 rounded-md px-1.5 py-0.5 text-sm/5 font-medium sm:text-xs/5 forced-colors:outline {} {}",
        color.as_classes(),
        class.unwrap_or_default()
    );

    rsx! {
        span {
            class: badge_classes,
            {children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct BadgeButtonProps {
    #[props(default = BadgeColor::Zinc)]
    pub color: BadgeColor,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub active: bool,
    pub onclick: EventHandler<()>,
    pub children: Element,
}

#[component]
pub fn BadgeButton(props: BadgeButtonProps) -> Element {
    let BadgeButtonProps {
        color,
        class,
        active,
        onclick,
        children,
    } = props;

    let wrapper_classes = format!(
        "group relative inline-flex rounded-md cursor-pointer focus:not-data-focus:outline-hidden data-focus:outline-2 data-focus:outline-offset-2 data-focus:outline-blue-500 {}",
        if active { "ring-2 ring-offset-1 ring-blue-500" } else { "" }
    );

    rsx! {
        button {
            class: wrapper_classes,
            onclick: move |_| onclick.call(()),
            Badge {
                color: color,
                class: class,
                {children}
            }
        }
    }
}
