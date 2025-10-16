use serde_json;

pub fn extract_json_from_text(text: &str) -> Option<serde_json::Value> {
    // Find the start of JSON (first { or [)
    let start_pos = text.find(|c| c == '{' || c == '[')?;
    
    // Extract JSON using brace matching
    if let Some(json_str) = extract_json_string_from_position(text, start_pos) {
        serde_json::from_str(&json_str).ok()
    } else {
        None
    }
}

pub fn extract_all_json_from_text(text: &str) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    let mut pos = 0;
    
    while pos < text.len() {
        // Find next JSON start
        if let Some(start_pos) = text[pos..].find(|c| c == '{' || c == '[') {
            let absolute_start = pos + start_pos;
            if let Some(json_str) = extract_json_string_from_position(text, absolute_start) {
                if let Ok(json_value) = serde_json::from_str(&json_str) {
                    results.push(json_value);
                }
                pos = absolute_start + json_str.len();
            } else {
                pos = absolute_start + 1;
            }
        } else {
            break;
        }
    }
    
    results
}

pub fn parse_json(json_str: &str) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(json_str)
}

fn extract_json_string_from_position(text: &str, start_pos: usize) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if start_pos >= chars.len() {
        return None;
    }
    
    let start_char = chars[start_pos];
    let end_char = match start_char {
        '{' => '}',
        '[' => ']',
        _ => return None,
    };
    
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escape_next = false;
    
    for (i, &ch) in chars.iter().enumerate().skip(start_pos) {
        if escape_next {
            escape_next = false;
            continue;
        }
        
        if ch == '\\' {
            escape_next = true;
            continue;
        }
        
        if ch == '"' && !escape_next {
            in_string = !in_string;
            continue;
        }
        
        if !in_string {
            if ch == start_char {
                brace_count += 1;
            } else if ch == end_char {
                brace_count -= 1;
                if brace_count == 0 {
                    return Some(text[start_pos..=i].to_string());
                }
            }
        }
    }
    
    None
}


