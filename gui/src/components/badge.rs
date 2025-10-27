use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BadgeColor {
    #[allow(dead_code)]
    Red,
    #[allow(dead_code)]
    Orange,
    Amber,
    #[allow(dead_code)]
    Yellow,
    #[allow(dead_code)]
    Lime,
    #[allow(dead_code)]
    Green,
    Emerald,
    #[allow(dead_code)]
    Teal,
    #[allow(dead_code)]
    Cyan,
    #[allow(dead_code)]
    Sky,
    Blue,
    #[allow(dead_code)]
    Indigo,
    #[allow(dead_code)]
    Violet,
    #[allow(dead_code)]
    Purple,
    #[allow(dead_code)]
    Fuchsia,
    #[allow(dead_code)]
    Pink,
    #[allow(dead_code)]
    Rose,
    Zinc,
}

impl BadgeColor {
    #[allow(clippy::wrong_self_convention)]
    fn to_classes(&self) -> &'static str {
        match self {
            BadgeColor::Red => {
                "bg-red-500/15 text-red-700 group-data-hover:bg-red-500/25 dark:bg-red-500/10 dark:text-red-400 dark:group-data-hover:bg-red-500/20"
            }
            BadgeColor::Orange => {
                "bg-orange-500/15 text-orange-700 group-data-hover:bg-orange-500/25 dark:bg-orange-500/10 dark:text-orange-400 dark:group-data-hover:bg-orange-500/20"
            }
            BadgeColor::Amber => {
                "bg-amber-400/20 text-amber-700 group-data-hover:bg-amber-400/30 dark:bg-amber-400/10 dark:text-amber-400 dark:group-data-hover:bg-amber-400/15"
            }
            BadgeColor::Yellow => {
                "bg-yellow-400/20 text-yellow-700 group-data-hover:bg-yellow-400/30 dark:bg-yellow-400/10 dark:text-yellow-300 dark:group-data-hover:bg-yellow-400/15"
            }
            BadgeColor::Lime => {
                "bg-lime-400/20 text-lime-700 group-data-hover:bg-lime-400/30 dark:bg-lime-400/10 dark:text-lime-300 dark:group-data-hover:bg-lime-400/15"
            }
            BadgeColor::Green => {
                "bg-green-500/15 text-green-700 group-data-hover:bg-green-500/25 dark:bg-green-500/10 dark:text-green-400 dark:group-data-hover:bg-green-500/20"
            }
            BadgeColor::Emerald => {
                "bg-emerald-500/15 text-emerald-700 group-data-hover:bg-emerald-500/25 dark:bg-emerald-500/10 dark:text-emerald-400 dark:group-data-hover:bg-emerald-500/20"
            }
            BadgeColor::Teal => {
                "bg-teal-500/15 text-teal-700 group-data-hover:bg-teal-500/25 dark:bg-teal-500/10 dark:text-teal-300 dark:group-data-hover:bg-teal-500/20"
            }
            BadgeColor::Cyan => {
                "bg-cyan-400/20 text-cyan-700 group-data-hover:bg-cyan-400/30 dark:bg-cyan-400/10 dark:text-cyan-300 dark:group-data-hover:bg-cyan-400/15"
            }
            BadgeColor::Sky => {
                "bg-sky-500/15 text-sky-700 group-data-hover:bg-sky-500/25 dark:bg-sky-500/10 dark:text-sky-300 dark:group-data-hover:bg-sky-500/20"
            }
            BadgeColor::Blue => {
                "bg-blue-500/15 text-blue-700 group-data-hover:bg-blue-500/25 dark:text-blue-400 dark:group-data-hover:bg-blue-500/25"
            }
            BadgeColor::Indigo => {
                "bg-indigo-500/15 text-indigo-700 group-data-hover:bg-indigo-500/25 dark:text-indigo-400 dark:group-data-hover:bg-indigo-500/20"
            }
            BadgeColor::Violet => {
                "bg-violet-500/15 text-violet-700 group-data-hover:bg-violet-500/25 dark:text-violet-400 dark:group-data-hover:bg-violet-500/20"
            }
            BadgeColor::Purple => {
                "bg-purple-500/15 text-purple-700 group-data-hover:bg-purple-500/25 dark:text-purple-400 dark:group-data-hover:bg-purple-500/20"
            }
            BadgeColor::Fuchsia => {
                "bg-fuchsia-400/15 text-fuchsia-700 group-data-hover:bg-fuchsia-400/25 dark:bg-fuchsia-400/10 dark:text-fuchsia-400 dark:group-data-hover:bg-fuchsia-400/20"
            }
            BadgeColor::Pink => {
                "bg-pink-400/15 text-pink-700 group-data-hover:bg-pink-400/25 dark:bg-pink-400/10 dark:text-pink-400 dark:group-data-hover:bg-pink-400/20"
            }
            BadgeColor::Rose => {
                "bg-rose-400/15 text-rose-700 group-data-hover:bg-rose-400/25 dark:bg-rose-400/10 dark:text-rose-400 dark:group-data-hover:bg-rose-400/20"
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
        color.to_classes(),
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
