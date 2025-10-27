//! System templates integration tests
//!
//! Tests that verify system templates load correctly from real files
//! in the /system directory at workspace root

use std::path::PathBuf;

#[tokio::test]
async fn test_system_workflows_directory_exists() {
    // Get workspace root (parent of core directory)
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let workflows_dir = workspace_root.join("system/workflows");

    assert!(
        workflows_dir.exists(),
        "system/workflows directory must exist at {:?}",
        workflows_dir
    );
}

#[tokio::test]
async fn test_system_prompts_directory_exists() {
    // Get workspace root (parent of core directory)
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let prompts_dir = workspace_root.join("system/prompts");

    assert!(
        prompts_dir.exists(),
        "system/prompts directory must exist at {:?}",
        prompts_dir
    );
}

#[tokio::test]
async fn test_system_workflows_file_count() {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let workflows_dir = workspace_root.join("system/workflows");

    let json_count = std::fs::read_dir(&workflows_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .count();

    assert!(
        json_count >= 3,
        "Expected at least 3 system workflows, found {}",
        json_count
    );
}

#[tokio::test]
async fn test_system_prompts_file_count() {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let prompts_dir = workspace_root.join("system/prompts");

    let json_count = std::fs::read_dir(&prompts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .count();

    assert!(
        json_count >= 3,
        "Expected at least 3 system prompts, found {}",
        json_count
    );
}

#[tokio::test]
async fn test_system_workflows_valid_json() {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let workflows_dir = workspace_root.join("system/workflows");

    for entry in std::fs::read_dir(&workflows_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            let content = std::fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("Failed to read file: {:?}", path);
            });

            let json: serde_json::Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Invalid JSON in {:?}", path));

            // Verify required fields
            assert!(
                json["id"].is_string(),
                "Missing or invalid 'id' in {:?}",
                path
            );
            assert!(
                json["name"].is_string(),
                "Missing or invalid 'name' in {:?}",
                path
            );
            assert!(
                json["version"].is_string(),
                "Missing or invalid 'version' in {:?}",
                path
            );
            assert!(
                json["content"].is_object(),
                "Missing or invalid 'content' (must be object) in {:?}",
                path
            );
            assert!(
                json["description"].is_string() || json["description"].is_null(),
                "Invalid 'description' (must be string or null) in {:?}",
                path
            );
        }
    }
}

#[tokio::test]
async fn test_system_prompts_valid_json() {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let prompts_dir = workspace_root.join("system/prompts");

    for entry in std::fs::read_dir(&prompts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            let content = std::fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("Failed to read file: {:?}", path);
            });

            let json: serde_json::Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Invalid JSON in {:?}", path));

            // Verify required fields
            assert!(
                json["id"].is_string(),
                "Missing or invalid 'id' in {:?}",
                path
            );
            assert!(
                json["name"].is_string(),
                "Missing or invalid 'name' in {:?}",
                path
            );
            assert!(
                json["version"].is_string(),
                "Missing or invalid 'version' in {:?}",
                path
            );
            assert!(
                json["content"].is_string(),
                "Missing or invalid 'content' (must be string) in {:?}",
                path
            );
            assert!(
                json["template"].is_string(),
                "Missing or invalid 'template' (must be string) in {:?}",
                path
            );
            assert!(
                json["description"].is_string() || json["description"].is_null(),
                "Invalid 'description' (must be string or null) in {:?}",
                path
            );
        }
    }
}

#[tokio::test]
async fn test_system_workflows_have_system_prefix() {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let workflows_dir = workspace_root.join("system/workflows");

    for entry in std::fs::read_dir(&workflows_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            let content = std::fs::read_to_string(&path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&content).unwrap();

            let id = json["id"].as_str().unwrap();

            assert!(
                id.starts_with("system:"),
                "System workflow ID '{}' in {:?} must start with 'system:'",
                id,
                path
            );
        }
    }
}

#[tokio::test]
async fn test_system_prompts_have_system_prefix() {
    let workspace = env!("CARGO_MANIFEST_DIR");
    let workspace_path = PathBuf::from(workspace);
    let workspace_root = workspace_path.parent().unwrap();
    let prompts_dir = workspace_root.join("system/prompts");

    for entry in std::fs::read_dir(&prompts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            let content = std::fs::read_to_string(&path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&content).unwrap();

            let id = json["id"].as_str().unwrap();

            assert!(
                id.starts_with("system:"),
                "System prompt ID '{}' in {:?} must start with 'system:'",
                id,
                path
            );
        }
    }
}
