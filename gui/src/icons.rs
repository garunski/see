use dioxus::prelude::*;

/// Icon component that renders SVG icons
#[component]
pub fn Icon(
    name: String,
    class: Option<String>,
    size: Option<String>,
    variant: Option<String>,
) -> Element {
    let icon_svg = get_icon_svg(&name);
    let class = class.unwrap_or_default();
    let size = size.unwrap_or_else(|| "w-4 h-4".to_string());

    rsx! {
        div {
            class: format!("{} {}", size, class),
            dangerous_inner_html: icon_svg
        }
    }
}

/// Get the SVG content for an icon by name
fn get_icon_svg(name: &str) -> String {
    match name {
        "home" => include_str!("../assets/icons/home-outline.svg").to_string(),
        "upload" => include_str!("../assets/icons/upload-outline.svg").to_string(),
        "workflows" => include_str!("../assets/icons/workflows-outline.svg").to_string(),
        "history" => include_str!("../assets/icons/history-outline.svg").to_string(),
        "prompts" => include_str!("../assets/icons/prompts-outline.svg").to_string(),
        "settings" => include_str!("../assets/icons/settings-outline.svg").to_string(),
        "plus" => include_str!("../assets/icons/plus-outline.svg").to_string(),
        "x" => include_str!("../assets/icons/x-outline.svg").to_string(),
        "chevron_left" => include_str!("../assets/icons/chevron_left-outline.svg").to_string(),
        "chevron_right" => include_str!("../assets/icons/chevron_right-outline.svg").to_string(),
        "arrow_left" => include_str!("../assets/icons/arrow_left-outline.svg").to_string(),
        "trash" => include_str!("../assets/icons/trash-outline.svg").to_string(),
        "exclamation_circle" => {
            include_str!("../assets/icons/exclamation_circle-outline.svg").to_string()
        }
        "play" => include_str!("../assets/icons/play-outline.svg").to_string(),
        "stop" => include_str!("../assets/icons/stop-outline.svg").to_string(),
        "bars_3" => include_str!("../assets/icons/bars_3-outline.svg").to_string(),
        "check_circle" => include_str!("../assets/icons/check_circle-outline.svg").to_string(),
        "save" => include_str!("../assets/icons/save-outline.svg").to_string(),
        "pause" => include_str!("../assets/icons/pause-outline.svg").to_string(),
        "copy" => include_str!("../assets/icons/copy-outline.svg").to_string(),
        _ => {
            // Fallback to a default icon or empty string
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor"><path d="M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zM10 6a4 4 0 100 8 4 4 0 000-8z" /></svg>"#.to_string()
        }
    }
}
