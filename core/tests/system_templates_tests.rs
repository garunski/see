//! System templates integration tests
//!
//! Tests that verify system templates are correctly embedded into the binary
//! and can be loaded and parsed properly.

use s_e_e_core::embedded_data;

#[tokio::test]
async fn test_embedded_workflows_count() {
    let workflows = embedded_data::get_default_workflows();
    assert_eq!(
        workflows.len(),
        4,
        "Expected exactly 4 embedded workflows, found {}",
        workflows.len()
    );
}

#[tokio::test]
async fn test_embedded_prompts_count() {
    let prompts = embedded_data::get_default_prompts();
    assert_eq!(
        prompts.len(),
        3,
        "Expected exactly 3 embedded prompts, found {}",
        prompts.len()
    );
}

#[tokio::test]
async fn test_embedded_workflows_parse() {
    let workflows = embedded_data::get_default_workflows();

    for (filename, content) in workflows {
        let json: serde_json::Value = serde_json::from_str(content)
            .unwrap_or_else(|_| panic!("Failed to parse workflow JSON in {}", filename));

        assert!(!json.is_null(), "Workflow {} should not be null", filename);
    }
}

#[tokio::test]
async fn test_embedded_prompts_parse() {
    let prompts = embedded_data::get_default_prompts();

    for (filename, content) in prompts {
        let json: serde_json::Value = serde_json::from_str(content)
            .unwrap_or_else(|_| panic!("Failed to parse prompt JSON in {}", filename));

        assert!(!json.is_null(), "Prompt {} should not be null", filename);
    }
}

#[tokio::test]
async fn test_embedded_workflows_valid() {
    let workflows = embedded_data::get_default_workflows();

    for (filename, content) in workflows {
        let json: serde_json::Value = serde_json::from_str(content)
            .unwrap_or_else(|_| panic!("Failed to parse workflow JSON in {}", filename));

        // Verify required fields
        assert!(
            json["id"].is_string(),
            "Missing or invalid 'id' in {}",
            filename
        );
        assert!(
            json["name"].is_string(),
            "Missing or invalid 'name' in {}",
            filename
        );
        assert!(
            json["version"].is_string(),
            "Missing or invalid 'version' in {}",
            filename
        );
        assert!(
            json["content"].is_object(),
            "Missing or invalid 'content' (must be object) in {}",
            filename
        );
        assert!(
            json["description"].is_string() || json["description"].is_null(),
            "Invalid 'description' (must be string or null) in {}",
            filename
        );
    }
}

#[tokio::test]
async fn test_embedded_prompts_valid() {
    let prompts = embedded_data::get_default_prompts();

    for (filename, content) in prompts {
        let json: serde_json::Value = serde_json::from_str(content)
            .unwrap_or_else(|_| panic!("Failed to parse prompt JSON in {}", filename));

        // Verify required fields match the Prompt model
        assert!(
            json["id"].is_string(),
            "Missing or invalid 'id' in {}",
            filename
        );
        assert!(
            json["name"].is_string(),
            "Missing or invalid 'name' in {}",
            filename
        );
        assert!(
            json["content"].is_string(),
            "Missing or invalid 'content' (must be string) in {}",
            filename
        );
    }
}

#[tokio::test]
async fn test_embedded_workflows_have_system_prefix() {
    let workflows = embedded_data::get_default_workflows();

    for (filename, content) in workflows {
        let json: serde_json::Value = serde_json::from_str(content).unwrap();
        let id = json["id"].as_str().unwrap();

        assert!(
            id.starts_with("system:"),
            "System workflow ID '{}' in {} must start with 'system:'",
            id,
            filename
        );
    }
}

#[tokio::test]
async fn test_embedded_prompts_have_system_prefix() {
    let prompts = embedded_data::get_default_prompts();

    for (filename, content) in prompts {
        let json: serde_json::Value = serde_json::from_str(content).unwrap();
        let id = json["id"].as_str().unwrap();

        assert!(
            id.starts_with("system:"),
            "System prompt ID '{}' in {} must start with 'system:'",
            id,
            filename
        );
    }
}

#[tokio::test]
async fn test_embedded_workflows_filenames() {
    let workflows = embedded_data::get_default_workflows();
    let filenames: Vec<&str> = workflows.iter().map(|(name, _)| *name).collect();

    assert!(
        filenames.contains(&"code-review.json"),
        "Missing code-review.json workflow"
    );
    assert!(
        filenames.contains(&"deploy-app.json"),
        "Missing deploy-app.json workflow"
    );
    assert!(
        filenames.contains(&"setup-project.json"),
        "Missing setup-project.json workflow"
    );
    assert!(
        filenames.contains(&"user-input-sample.json"),
        "Missing user-input-sample.json workflow"
    );
}

#[tokio::test]
async fn test_embedded_prompts_filenames() {
    let prompts = embedded_data::get_default_prompts();
    let filenames: Vec<&str> = prompts.iter().map(|(name, _)| *name).collect();

    assert!(
        filenames.contains(&"bug-fix.json"),
        "Missing bug-fix.json prompt"
    );
    assert!(
        filenames.contains(&"code-review.json"),
        "Missing code-review.json prompt"
    );
    assert!(
        filenames.contains(&"documentation.json"),
        "Missing documentation.json prompt"
    );
}
