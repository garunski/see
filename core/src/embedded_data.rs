pub const CODE_REVIEW_WORKFLOW: &str = include_str!("../initial_data/workflows/code-review.json");
pub const DEPLOY_APP_WORKFLOW: &str = include_str!("../initial_data/workflows/deploy-app.json");
pub const SETUP_PROJECT_WORKFLOW: &str =
    include_str!("../initial_data/workflows/setup-project.json");
pub const USER_INPUT_SAMPLE_WORKFLOW: &str =
    include_str!("../initial_data/workflows/user-input-sample.json");

pub const BUG_FIX_PROMPT: &str = include_str!("../initial_data/prompts/bug-fix.json");
pub const CODE_REVIEW_PROMPT: &str = include_str!("../initial_data/prompts/code-review.json");
pub const DOCUMENTATION_PROMPT: &str = include_str!("../initial_data/prompts/documentation.json");

pub fn get_default_workflows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("code-review.json", CODE_REVIEW_WORKFLOW),
        ("deploy-app.json", DEPLOY_APP_WORKFLOW),
        ("setup-project.json", SETUP_PROJECT_WORKFLOW),
        ("user-input-sample.json", USER_INPUT_SAMPLE_WORKFLOW),
    ]
}

pub fn get_default_prompts() -> Vec<(&'static str, &'static str)> {
    vec![
        ("bug-fix.json", BUG_FIX_PROMPT),
        ("code-review.json", CODE_REVIEW_PROMPT),
        ("documentation.json", DOCUMENTATION_PROMPT),
    ]
}
