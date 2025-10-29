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
        "executions" => include_str!("../assets/icons/executions-outline.svg").to_string(),
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
        "code_bracket" => include_str!("../assets/icons/code_bracket-outline.svg").to_string(),
        "computer-desktop" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>"#.to_string(),
        "sun" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>"#.to_string(),
        "moon" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>"#.to_string(),
        _ => {
            // Fallback to a default icon or empty string
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor"><path d="M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zM10 6a4 4 0 100 8 4 4 0 000-8z" /></svg>"#.to_string()
        }
    }
}
