use simple_workflow_app::json_parser;

#[test]
fn test_parse_pure_json_object() {
    let json_str = r#"{"name": "test", "value": 42}"#;
    let result = json_parser::parse_json(json_str);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["name"], "test");
    assert_eq!(value["value"], 42);
}

#[test]
fn test_parse_pure_json_array() {
    let json_str = r#"[1, 2, 3, "hello"]"#;
    let result = json_parser::parse_json(json_str);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_array());
    assert_eq!(value[0], 1);
    assert_eq!(value[3], "hello");
}

#[test]
fn test_extract_json_from_text_with_surrounding_content() {
    let text = "Here is some text before JSON: {\"name\": \"test\", \"value\": 42} and more text after";
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert_eq!(value["name"], "test");
    assert_eq!(value["value"], 42);
}

#[test]
fn test_extract_json_from_markdown_code_block() {
    let text = "Here's some JSON:\n```json\n{\"key\": \"value\"}\n```\nMore text";
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert_eq!(value["key"], "value");
}

#[test]
fn test_extract_multiple_json_objects() {
    let text = "First: {\"a\": 1} Second: [1, 2, 3] Third: {\"b\": 2}";
    let results = json_parser::extract_all_json_from_text(text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0]["a"], 1);
    assert_eq!(results[1][0], 1);
    assert_eq!(results[2]["b"], 2);
}

#[test]
fn test_handle_malformed_json_gracefully() {
    let text = "Here is malformed JSON: {\"name\": \"test\", \"value\":} and more text";
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_none());
}

#[test]
fn test_handle_text_with_no_json() {
    let text = "This is just plain text with no JSON at all";
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_none());
    
    let results = json_parser::extract_all_json_from_text(text);
    assert!(results.is_empty());
}

#[test]
fn test_handle_nested_json_structures() {
    let text = r#"{"outer": {"inner": {"deep": "value"}}, "array": [1, 2, 3]}"#;
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert_eq!(value["outer"]["inner"]["deep"], "value");
    assert_eq!(value["array"][0], 1);
}

#[test]
fn test_handle_escaped_characters_in_json() {
    let text = r#"{"message": "He said \"Hello\" and left", "path": "C:\\Users\\test"}"#;
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert_eq!(value["message"], "He said \"Hello\" and left");
    assert_eq!(value["path"], "C:\\Users\\test");
}

#[test]
fn test_handle_empty_string() {
    let result = json_parser::extract_json_from_text("");
    assert!(result.is_none());
    
    let results = json_parser::extract_all_json_from_text("");
    assert!(results.is_empty());
}

#[test]
fn test_handle_whitespace_only() {
    let text = "   \n\t  \n  ";
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_none());
    
    let results = json_parser::extract_all_json_from_text(text);
    assert!(results.is_empty());
}

#[test]
fn test_handle_json_with_whitespace() {
    let text = "  {\n    \"name\": \"test\",\n    \"value\": 42\n  }  ";
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert_eq!(value["name"], "test");
    assert_eq!(value["value"], 42);
}

#[test]
fn test_handle_json_array_with_objects() {
    let text = r#"[{"id": 1, "name": "first"}, {"id": 2, "name": "second"}]"#;
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert!(value.is_array());
    assert_eq!(value[0]["id"], 1);
    assert_eq!(value[1]["name"], "second");
}

#[test]
fn test_handle_complex_nested_structures() {
    let text = r#"{"users": [{"name": "Alice", "scores": [95, 87, 92]}, {"name": "Bob", "scores": [78, 91, 85]}], "metadata": {"total": 2}}"#;
    let result = json_parser::extract_json_from_text(text);
    assert!(result.is_some());
    let value = result.unwrap();
    assert_eq!(value["users"][0]["name"], "Alice");
    assert_eq!(value["users"][0]["scores"][0], 95);
    assert_eq!(value["metadata"]["total"], 2);
}

#[test]
fn test_parse_json_error_handling() {
    let invalid_json = r#"{"name": "test", "value":}"#;
    let result = json_parser::parse_json(invalid_json);
    assert!(result.is_err());
}
