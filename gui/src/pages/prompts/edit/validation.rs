use s_e_e_core::Prompt;

pub fn validate_prompt_fields(prompt_id: &str, name: &str, content: &str) -> Result<(), String> {
    if prompt_id.trim().is_empty() {
        return Err("ID is required".to_string());
    }
    if name.trim().is_empty() {
        return Err("Name is required".to_string());
    }
    if content.trim().is_empty() {
        return Err("Content is required".to_string());
    }
    Ok(())
}

pub fn create_prompt_from_fields(
    prompt_id: String,
    name: String,
    content: String,
) -> Prompt {
    let now = chrono::Utc::now();
    let content_str = content.trim().to_string();
    Prompt {
        id: prompt_id.trim().to_string(),
        name: name.trim().to_string(),
        content: content_str,
        created_at: now,
    }
}
