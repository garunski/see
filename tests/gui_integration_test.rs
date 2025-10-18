use dioxus::prelude::*;
use simple_workflow_app::components::WorkflowInfoCard;
use simple_workflow_app::{WorkflowResult, AuditEntry};

// Helper function to create a mock WorkflowResult for testing
fn create_mock_workflow_result(success: bool) -> WorkflowResult {
    WorkflowResult {
        success,
        workflow_name: "Test Workflow".to_string(),
        task_count: 3,
        final_context: serde_json::json!({
            "test_key": "test_value",
            "count": 42
        }),
        audit_trail: vec![
            AuditEntry {
                task_id: "task_1".to_string(),
                status: "200".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                changes_count: 2,
            },
        ],
        errors: if success {
            Vec::new()
        } else {
            vec!["Test error message".to_string()]
        },
        output_logs: vec![
            "Loading workflow...".to_string(),
            "Workflow complete!".to_string(),
        ],
    }
}

#[component]
fn TestWorkflowInfoCardSuccess() -> Element {
    let result = create_mock_workflow_result(true);
    let result_signal = use_signal(|| result);
    rsx! {
        WorkflowInfoCard {
            result: result_signal
        }
    }
}

#[component]
fn TestWorkflowInfoCardFailure() -> Element {
    let result = create_mock_workflow_result(false);
    let result_signal = use_signal(|| result);
    rsx! {
        WorkflowInfoCard {
            result: result_signal
        }
    }
}

#[test]
fn test_workflow_info_card_renders_success_state() {
    // Test that WorkflowInfoCard component renders successfully with success state
    let mut dom = VirtualDom::new(TestWorkflowInfoCardSuccess);
    
    // Rebuild to get initial render
    dom.rebuild_in_place();
    
    // Render to HTML string
    let html = dioxus_ssr::render(&dom);
    
    // Verify the component rendered the workflow name
    assert!(html.contains("Test Workflow"), "Should contain workflow name, got: {}", html);
    
    // Verify it shows success status
    assert!(html.contains("Success"), "Should show Success status, got: {}", html);
    
    // Verify it shows the task count
    assert!(html.contains("3"), "Should show task count of 3, got: {}", html);
    
    // Verify the success checkmark is present
    assert!(html.contains("✓"), "Should contain success checkmark, got: {}", html);
}

#[test]
fn test_workflow_info_card_renders_failure_state() {
    // Test that WorkflowInfoCard component renders correctly with failure state
    let mut dom = VirtualDom::new(TestWorkflowInfoCardFailure);
    
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    
    // Verify the component rendered the workflow name
    assert!(html.contains("Test Workflow"), "Should contain workflow name, got: {}", html);
    
    // Verify it shows failure status
    assert!(html.contains("Failed"), "Should show Failed status, got: {}", html);
    
    // Verify the failure X mark is present
    assert!(html.contains("✕"), "Should contain failure X mark, got: {}", html);
}

#[test]
fn test_workflow_info_card_renders_all_fields() {
    // Test that all data fields are rendered in the component
    let mut dom = VirtualDom::new(TestWorkflowInfoCardSuccess);
    
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    
    // Verify workflow name section
    assert!(html.contains("Workflow Name"), "Should have Workflow Name label, got: {}", html);
    assert!(html.contains("Test Workflow"), "Should show the actual workflow name, got: {}", html);
    
    // Verify tasks section
    assert!(html.contains("Tasks"), "Should have Tasks label, got: {}", html);
    assert!(html.contains("3"), "Should show task count, got: {}", html);
    
    // Verify status section
    assert!(html.contains("Status"), "Should have Status label, got: {}", html);
    assert!(html.contains("Success") || html.contains("Failed"), "Should show status, got: {}", html);
}

#[test]
fn test_workflow_info_card_has_proper_structure() {
    // Test that the rendered HTML has the expected structure
    let mut dom = VirtualDom::new(TestWorkflowInfoCardSuccess);
    
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    
    // Verify it has div structure
    assert!(html.contains("<div"), "Should contain div elements, got: {}", html);
    
    // Verify it has the main heading
    assert!(html.contains("Workflow Results"), "Should have 'Workflow Results' heading, got: {}", html);
    
    // Verify it has grid layout for stats
    assert!(html.contains("grid"), "Should use grid layout, got: {}", html);
}
