

use core::*;

#[test]
fn test_default_workflows() {
    let defaults = crate::api::defaults::get_default_workflows();

    assert!(!defaults.is_empty(), "Should have default workflows");


    for workflow in &defaults {
        assert!(workflow.is_default, "Default workflow should have is_default = true");
        assert!(!workflow.content.is_empty(), "Default workflow should have content");
    }


    let ids: Vec<&String> = defaults.iter().map(|w| &w.id).collect();
    assert!(ids.contains(&&"default-simple".to_string()));
    assert!(ids.contains(&&"default-parallel".to_string()));
    assert!(ids.contains(&&"default-nested".to_string()));
}

#[test]
fn test_default_workflows_content_valid() {
    let defaults = crate::api::defaults::get_default_workflows();

    for workflow in &defaults {

        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&workflow.content);
        assert!(parse_result.is_ok(), "Default workflow content should be valid JSON: {}", workflow.id);

        let json = parse_result.unwrap();


        assert!(json.get("id").is_some(), "Default workflow should have id field: {}", workflow.id);
        assert!(json.get("name").is_some(), "Default workflow should have name field: {}", workflow.id);
        assert!(json.get("tasks").is_some(), "Default workflow should have tasks field: {}", workflow.id);
    }
}

#[test]
fn test_default_workflows_unique_ids() {
    let defaults = crate::api::defaults::get_default_workflows();
    let mut ids = std::collections::HashSet::new();

    for workflow in &defaults {
        assert!(ids.insert(&workflow.id), "Default workflow IDs should be unique: {}", workflow.id);
    }
}
