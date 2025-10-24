use dioxus::prelude::*;

/// Icon component that renders SVG icons
#[component]
pub fn Icon(
    name: String,
    class: Option<String>,
    size: Option<String>,
    variant: Option<String>,
) -> Element {
    let icon_svg = get_icon_svg(&name, variant.as_deref().unwrap_or("outline"));
    let class = class.unwrap_or_default();
    let size = size.unwrap_or_else(|| "w-4 h-4".to_string());

    rsx! {
        div {
            class: format!("{} {}", size, class),
            dangerous_inner_html: icon_svg
        }
    }
}

/// Get the SVG content for an icon by name and variant
fn get_icon_svg(name: &str, variant: &str) -> String {
    match (name, variant) {
        ("home", "solid") => include_str!("../assets/icons/home-solid.svg").to_string(),
        ("home", "outline") => include_str!("../assets/icons/home-outline.svg").to_string(),
        ("upload", "solid") => include_str!("../assets/icons/upload-solid.svg").to_string(),
        ("upload", "outline") => include_str!("../assets/icons/upload-outline.svg").to_string(),
        ("workflows", "solid") => include_str!("../assets/icons/workflows-solid.svg").to_string(),
        ("workflows", "outline") => {
            include_str!("../assets/icons/workflows-outline.svg").to_string()
        }
        ("history", "solid") => include_str!("../assets/icons/history-solid.svg").to_string(),
        ("history", "outline") => include_str!("../assets/icons/history-outline.svg").to_string(),
        ("prompts", "solid") => include_str!("../assets/icons/prompts-solid.svg").to_string(),
        ("prompts", "outline") => include_str!("../assets/icons/prompts-outline.svg").to_string(),
        ("settings", "solid") => include_str!("../assets/icons/settings-solid.svg").to_string(),
        ("settings", "outline") => include_str!("../assets/icons/settings-outline.svg").to_string(),
        ("plus", "solid") => include_str!("../assets/icons/plus-solid.svg").to_string(),
        ("plus", "outline") => include_str!("../assets/icons/plus-outline.svg").to_string(),
        ("x", "solid") => include_str!("../assets/icons/x-solid.svg").to_string(),
        ("x", "outline") => include_str!("../assets/icons/x-outline.svg").to_string(),
        ("chevron_left", "solid") => {
            include_str!("../assets/icons/chevron_left-solid.svg").to_string()
        }
        ("chevron_left", "outline") => {
            include_str!("../assets/icons/chevron_left-outline.svg").to_string()
        }
        ("chevron_right", "solid") => {
            include_str!("../assets/icons/chevron_right-solid.svg").to_string()
        }
        ("chevron_right", "outline") => {
            include_str!("../assets/icons/chevron_right-outline.svg").to_string()
        }
        ("arrow_left", "solid") => include_str!("../assets/icons/arrow_left-solid.svg").to_string(),
        ("arrow_left", "outline") => {
            include_str!("../assets/icons/arrow_left-outline.svg").to_string()
        }
        ("trash", "solid") => include_str!("../assets/icons/trash-solid.svg").to_string(),
        ("trash", "outline") => include_str!("../assets/icons/trash-outline.svg").to_string(),
        ("exclamation_circle", "solid") => {
            include_str!("../assets/icons/exclamation_circle-solid.svg").to_string()
        }
        ("exclamation_circle", "outline") => {
            include_str!("../assets/icons/exclamation_circle-outline.svg").to_string()
        }
        ("play", "solid") => include_str!("../assets/icons/play-solid.svg").to_string(),
        ("play", "outline") => include_str!("../assets/icons/play-outline.svg").to_string(),
        ("stop", "solid") => include_str!("../assets/icons/stop-solid.svg").to_string(),
        ("stop", "outline") => include_str!("../assets/icons/stop-outline.svg").to_string(),
        ("bars_3", "solid") => include_str!("../assets/icons/bars_3-solid.svg").to_string(),
        ("bars_3", "outline") => include_str!("../assets/icons/bars_3-outline.svg").to_string(),
        _ => {
            // Fallback to a default icon or empty string
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor"><path d="M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zM10 6a4 4 0 100 8 4 4 0 000-8z" /></svg>"#.to_string()
        }
    }
}
