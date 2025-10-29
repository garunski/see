//! Embedded system templates (workflows and prompts)
//!
//! These templates are baked into the binary at compile time using `include_str!`.
//! This eliminates runtime dependencies on the filesystem and ensures templates
//! are always available.

// Workflows - embedded at compile time
pub const CODE_REVIEW_WORKFLOW: &str =
    include_str!("../initial_data/workflows/code-review.json");
pub const DEPLOY_APP_WORKFLOW: &str = include_str!("../initial_data/workflows/deploy-app.json");
pub const SETUP_PROJECT_WORKFLOW: &str =
    include_str!("../initial_data/workflows/setup-project.json");
pub const USER_INPUT_SAMPLE_WORKFLOW: &str =
    include_str!("../initial_data/workflows/user-input-sample.json");

// Prompts - embedded at compile time
pub const BUG_FIX_PROMPT: &str = include_str!("../initial_data/prompts/bug-fix.json");
pub const CODE_REVIEW_PROMPT: &str = include_str!("../initial_data/prompts/code-review.json");
pub const DOCUMENTATION_PROMPT: &str = include_str!("../initial_data/prompts/documentation.json");

/// Get all embedded workflows as (filename, json_content) pairs
///
/// Returns a vector of tuples containing the filename and the embedded JSON content.
/// This is used during initial data population to load system workflows into the database.
pub fn get_default_workflows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("code-review.json", CODE_REVIEW_WORKFLOW),
        ("deploy-app.json", DEPLOY_APP_WORKFLOW),
        ("setup-project.json", SETUP_PROJECT_WORKFLOW),
        ("user-input-sample.json", USER_INPUT_SAMPLE_WORKFLOW),
    ]
}

/// Get all embedded prompts as (filename, json_content) pairs
///
/// Returns a vector of tuples containing the filename and the embedded JSON content.
/// This is used during initial data population to load system prompts into the database.
pub fn get_default_prompts() -> Vec<(&'static str, &'static str)> {
    vec![
        ("bug-fix.json", BUG_FIX_PROMPT),
        ("code-review.json", CODE_REVIEW_PROMPT),
        ("documentation.json", DOCUMENTATION_PROMPT),
    ]
}

