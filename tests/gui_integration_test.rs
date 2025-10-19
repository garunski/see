use dioxus::prelude::*;
use simple_workflow_app::components::{
    ContextPanel, ErrorsPanel, ExecutionStatus, OutputLogsPanel, Sidebar, Toast, WorkflowInfoCard,
};
use simple_workflow_app::{AuditEntry, WorkflowResult};

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
        audit_trail: vec![AuditEntry {
            task_id: "task_1".to_string(),
            status: "200".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            changes_count: 2,
        }],
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

// ===== WORKFLOW INFO CARD TESTS =====

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
    let mut dom = VirtualDom::new(TestWorkflowInfoCardSuccess);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Test Workflow"),
        "Should contain workflow name, got: {}",
        html
    );
    assert!(
        html.contains("Success"),
        "Should show Success status, got: {}",
        html
    );
    assert!(
        html.contains("3"),
        "Should show task count of 3, got: {}",
        html
    );
    assert!(
        html.contains("✓"),
        "Should contain success checkmark, got: {}",
        html
    );
}

#[test]
fn test_workflow_info_card_renders_failure_state() {
    let mut dom = VirtualDom::new(TestWorkflowInfoCardFailure);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Test Workflow"),
        "Should contain workflow name, got: {}",
        html
    );
    assert!(
        html.contains("Failed"),
        "Should show Failed status, got: {}",
        html
    );
    assert!(
        html.contains("✕"),
        "Should contain failure X mark, got: {}",
        html
    );
}

#[test]
fn test_workflow_info_card_renders_all_fields() {
    let mut dom = VirtualDom::new(TestWorkflowInfoCardSuccess);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Workflow Name"),
        "Should have Workflow Name label, got: {}",
        html
    );
    assert!(
        html.contains("Test Workflow"),
        "Should show the actual workflow name, got: {}",
        html
    );
    assert!(
        html.contains("Tasks"),
        "Should have Tasks label, got: {}",
        html
    );
    assert!(html.contains("3"), "Should show task count, got: {}", html);
    assert!(
        html.contains("Status"),
        "Should have Status label, got: {}",
        html
    );
    assert!(
        html.contains("Success") || html.contains("Failed"),
        "Should show status, got: {}",
        html
    );
}

#[test]
fn test_workflow_info_card_has_proper_structure() {
    let mut dom = VirtualDom::new(TestWorkflowInfoCardSuccess);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("<div"),
        "Should contain div elements, got: {}",
        html
    );
    assert!(
        html.contains("Workflow Results"),
        "Should have 'Workflow Results' heading, got: {}",
        html
    );
    assert!(
        html.contains("grid"),
        "Should use grid layout, got: {}",
        html
    );
}

// ===== ERRORS PANEL TESTS =====

#[component]
fn TestErrorsPanelWithErrors() -> Element {
    let errors = vec![
        "Error 1: Something went wrong".to_string(),
        "Error 2: Another issue occurred".to_string(),
    ];
    rsx! {
        ErrorsPanel {
            errors: errors
        }
    }
}

#[component]
fn TestErrorsPanelEmpty() -> Element {
    let errors = vec![];
    rsx! {
        ErrorsPanel {
            errors: errors
        }
    }
}

#[test]
fn test_errors_panel_renders_with_errors() {
    let mut dom = VirtualDom::new(TestErrorsPanelWithErrors);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Errors"),
        "Should have Errors heading, got: {}",
        html
    );
    assert!(
        html.contains("⚠️"),
        "Should contain warning icon, got: {}",
        html
    );
    assert!(
        html.contains("Error 1: Something went wrong"),
        "Should show first error, got: {}",
        html
    );
    assert!(
        html.contains("Error 2: Another issue occurred"),
        "Should show second error, got: {}",
        html
    );
    assert!(
        html.contains("bg-red-50"),
        "Should have red background styling, got: {}",
        html
    );
}

#[test]
fn test_errors_panel_renders_empty_when_no_errors() {
    let mut dom = VirtualDom::new(TestErrorsPanelEmpty);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    // Should render empty div when no errors
    assert!(
        !html.contains("Errors"),
        "Should not show Errors heading when empty, got: {}",
        html
    );
    assert!(
        !html.contains("⚠️"),
        "Should not show warning icon when empty, got: {}",
        html
    );
}

#[test]
fn test_errors_panel_has_proper_styling() {
    let mut dom = VirtualDom::new(TestErrorsPanelWithErrors);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("rounded-2xl"),
        "Should have rounded corners, got: {}",
        html
    );
    assert!(
        html.contains("font-mono"),
        "Should use monospace font for errors, got: {}",
        html
    );
    assert!(
        html.contains("space-y-3"),
        "Should have proper spacing, got: {}",
        html
    );
}

// ===== CONTEXT PANEL TESTS =====

#[component]
fn TestContextPanelCollapsed() -> Element {
    let context = serde_json::json!({
        "key1": "value1",
        "key2": 42,
        "nested": {
            "inner": "data"
        }
    });
    let show_context = false;
    let on_toggle = move |_| {};
    let on_copy = move |_| {};

    rsx! {
        ContextPanel {
            context: context,
            show_context: show_context,
            on_toggle: on_toggle,
            on_copy: on_copy
        }
    }
}

#[component]
fn TestContextPanelExpanded() -> Element {
    let context = serde_json::json!({
        "key1": "value1",
        "key2": 42
    });
    let show_context = true;
    let on_toggle = move |_| {};
    let on_copy = move |_| {};

    rsx! {
        ContextPanel {
            context: context,
            show_context: show_context,
            on_toggle: on_toggle,
            on_copy: on_copy
        }
    }
}

#[test]
fn test_context_panel_renders_collapsed() {
    let mut dom = VirtualDom::new(TestContextPanelCollapsed);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Final Context"),
        "Should have Final Context heading, got: {}",
        html
    );
    assert!(
        html.contains("📊"),
        "Should contain chart icon, got: {}",
        html
    );
    assert!(
        html.contains("▼"),
        "Should show collapsed arrow, got: {}",
        html
    );
    assert!(
        !html.contains("key1"),
        "Should not show context data when collapsed, got: {}",
        html
    );
}

#[test]
fn test_context_panel_renders_expanded() {
    let mut dom = VirtualDom::new(TestContextPanelExpanded);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Final Context"),
        "Should have Final Context heading, got: {}",
        html
    );
    assert!(
        html.contains("key1"),
        "Should show context data when expanded, got: {}",
        html
    );
    assert!(
        html.contains("value1"),
        "Should show context values, got: {}",
        html
    );
    assert!(
        html.contains("Copy Context"),
        "Should have copy button, got: {}",
        html
    );
    assert!(
        html.contains("rotate-180"),
        "Should show expanded arrow, got: {}",
        html
    );
}

#[test]
fn test_context_panel_has_proper_structure() {
    let mut dom = VirtualDom::new(TestContextPanelExpanded);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("rounded-2xl"),
        "Should have rounded corners, got: {}",
        html
    );
    assert!(
        html.contains("overflow-hidden"),
        "Should have overflow hidden, got: {}",
        html
    );
    assert!(
        html.contains("font-mono"),
        "Should use monospace font for JSON, got: {}",
        html
    );
    assert!(
        html.contains("whitespace-pre-wrap"),
        "Should preserve whitespace, got: {}",
        html
    );
}

// ===== OUTPUT LOGS PANEL TESTS =====

#[component]
fn TestOutputLogsPanelWithLogs() -> Element {
    let logs = vec![
        "Loading workflow...".to_string(),
        "Executing tasks...".to_string(),
        "Workflow complete!".to_string(),
    ];
    let show_logs = true;
    let on_toggle = move |_| {};
    let on_copy = move |_| {};

    rsx! {
        OutputLogsPanel {
            logs: logs,
            show_logs: show_logs,
            on_toggle: on_toggle,
            on_copy: on_copy
        }
    }
}

#[component]
fn TestOutputLogsPanelCollapsed() -> Element {
    let logs = vec!["Test log message".to_string()];
    let show_logs = false;
    let on_toggle = move |_| {};
    let on_copy = move |_| {};

    rsx! {
        OutputLogsPanel {
            logs: logs,
            show_logs: show_logs,
            on_toggle: on_toggle,
            on_copy: on_copy
        }
    }
}

#[component]
fn TestOutputLogsPanelEmpty() -> Element {
    let logs = vec![];
    let show_logs = false;
    let on_toggle = move |_| {};
    let on_copy = move |_| {};

    rsx! {
        OutputLogsPanel {
            logs: logs,
            show_logs: show_logs,
            on_toggle: on_toggle,
            on_copy: on_copy
        }
    }
}

#[test]
fn test_output_logs_panel_renders_with_logs() {
    let mut dom = VirtualDom::new(TestOutputLogsPanelWithLogs);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Execution Output"),
        "Should have Execution Output heading, got: {}",
        html
    );
    assert!(
        html.contains("📋"),
        "Should contain clipboard icon, got: {}",
        html
    );
    assert!(
        html.contains("(3 lines)"),
        "Should show log count, got: {}",
        html
    );
    assert!(
        html.contains("Loading workflow..."),
        "Should show log content, got: {}",
        html
    );
    assert!(
        html.contains("Copy Output"),
        "Should have copy button, got: {}",
        html
    );
}

#[test]
fn test_output_logs_panel_renders_collapsed() {
    let mut dom = VirtualDom::new(TestOutputLogsPanelCollapsed);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Execution Output"),
        "Should have Execution Output heading, got: {}",
        html
    );
    assert!(
        !html.contains("Test log message"),
        "Should not show log content when collapsed, got: {}",
        html
    );
    assert!(
        !html.contains("Copy Output"),
        "Should not show copy button when collapsed, got: {}",
        html
    );
}

#[test]
fn test_output_logs_panel_renders_empty() {
    let mut dom = VirtualDom::new(TestOutputLogsPanelEmpty);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    // Should render empty div when no logs
    assert!(
        !html.contains("Execution Output"),
        "Should not show heading when empty, got: {}",
        html
    );
    assert!(
        !html.contains("📋"),
        "Should not show icon when empty, got: {}",
        html
    );
}

#[test]
fn test_output_logs_panel_has_proper_styling() {
    let mut dom = VirtualDom::new(TestOutputLogsPanelWithLogs);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("max-h-80"),
        "Should have max height constraint, got: {}",
        html
    );
    assert!(
        html.contains("overflow-y-auto"),
        "Should be scrollable, got: {}",
        html
    );
    assert!(
        html.contains("font-mono"),
        "Should use monospace font, got: {}",
        html
    );
}

// ===== TOAST TESTS =====

#[component]
fn TestToastWithMessage() -> Element {
    let message = Some("Test toast message".to_string());
    let on_dismiss = move |_| {};

    rsx! {
        Toast {
            message: message,
            on_dismiss: on_dismiss
        }
    }
}

#[component]
fn TestToastEmpty() -> Element {
    let message = None;
    let on_dismiss = move |_| {};

    rsx! {
        Toast {
            message: message,
            on_dismiss: on_dismiss
        }
    }
}

#[test]
fn test_toast_renders_with_message() {
    let mut dom = VirtualDom::new(TestToastWithMessage);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Test toast message"),
        "Should show toast message, got: {}",
        html
    );
    assert!(
        html.contains("✕"),
        "Should have dismiss button, got: {}",
        html
    );
    assert!(
        html.contains("fixed top-6 right-6"),
        "Should be positioned fixed, got: {}",
        html
    );
    assert!(
        html.contains("z-50"),
        "Should have high z-index, got: {}",
        html
    );
}

#[test]
fn test_toast_renders_empty() {
    let mut dom = VirtualDom::new(TestToastEmpty);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    // Should render empty div when no message
    assert!(
        !html.contains("Test toast message"),
        "Should not show message when empty, got: {}",
        html
    );
    assert!(
        !html.contains("✕"),
        "Should not show dismiss button when empty, got: {}",
        html
    );
}

#[test]
fn test_toast_has_proper_styling() {
    let mut dom = VirtualDom::new(TestToastWithMessage);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("rounded-2xl"),
        "Should have rounded corners, got: {}",
        html
    );
    assert!(
        html.contains("shadow-xl"),
        "Should have shadow, got: {}",
        html
    );
    assert!(
        html.contains("animate-slide-in"),
        "Should have slide animation, got: {}",
        html
    );
}

// ===== SIDEBAR TESTS =====

#[component]
fn TestSidebarIdle() -> Element {
    let workflow_file = "test.json".to_string();
    let on_workflow_file_change = move |_| {};
    let is_picking_file = false;
    let on_pick_file = move |_| {};
    let dark_mode = false;
    let on_toggle_dark_mode = move |_| {};
    let execution_status = ExecutionStatus::Idle;
    let on_execute = move |_| {};

    rsx! {
        Sidebar {
            workflow_file: workflow_file,
            on_workflow_file_change: on_workflow_file_change,
            is_picking_file: is_picking_file,
            on_pick_file: on_pick_file,
            dark_mode: dark_mode,
            on_toggle_dark_mode: on_toggle_dark_mode,
            execution_status: execution_status,
            on_execute: on_execute
        }
    }
}

#[component]
fn TestSidebarRunning() -> Element {
    let workflow_file = "test.json".to_string();
    let on_workflow_file_change = move |_| {};
    let is_picking_file = false;
    let on_pick_file = move |_| {};
    let dark_mode = false;
    let on_toggle_dark_mode = move |_| {};
    let execution_status = ExecutionStatus::Running;
    let on_execute = move |_| {};

    rsx! {
        Sidebar {
            workflow_file: workflow_file,
            on_workflow_file_change: on_workflow_file_change,
            is_picking_file: is_picking_file,
            on_pick_file: on_pick_file,
            dark_mode: dark_mode,
            on_toggle_dark_mode: on_toggle_dark_mode,
            execution_status: execution_status,
            on_execute: on_execute
        }
    }
}

#[component]
fn TestSidebarDarkMode() -> Element {
    let workflow_file = "test.json".to_string();
    let on_workflow_file_change = move |_| {};
    let is_picking_file = false;
    let on_pick_file = move |_| {};
    let dark_mode = true;
    let on_toggle_dark_mode = move |_| {};
    let execution_status = ExecutionStatus::Idle;
    let on_execute = move |_| {};

    rsx! {
        Sidebar {
            workflow_file: workflow_file,
            on_workflow_file_change: on_workflow_file_change,
            is_picking_file: is_picking_file,
            on_pick_file: on_pick_file,
            dark_mode: dark_mode,
            on_toggle_dark_mode: on_toggle_dark_mode,
            execution_status: execution_status,
            on_execute: on_execute
        }
    }
}

#[test]
fn test_sidebar_renders_basic_elements() {
    let mut dom = VirtualDom::new(TestSidebarIdle);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Workflow Executor"),
        "Should have app title, got: {}",
        html
    );
    assert!(
        html.contains("⚡"),
        "Should have lightning icon, got: {}",
        html
    );
    assert!(
        html.contains("Execute and manage workflows"),
        "Should have subtitle, got: {}",
        html
    );
    assert!(
        html.contains("Workflow File"),
        "Should have file input label, got: {}",
        html
    );
    assert!(
        html.contains("Execute Workflow"),
        "Should have execute button, got: {}",
        html
    );
    assert!(
        html.contains("🚀"),
        "Should have rocket icon on execute button, got: {}",
        html
    );
}

#[test]
fn test_sidebar_renders_theme_toggle() {
    let mut dom = VirtualDom::new(TestSidebarIdle);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Light Mode"),
        "Should show light mode text, got: {}",
        html
    );
    assert!(
        html.contains("☀️"),
        "Should show sun icon for light mode, got: {}",
        html
    );
}

#[test]
fn test_sidebar_renders_dark_mode() {
    let mut dom = VirtualDom::new(TestSidebarDarkMode);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Dark Mode"),
        "Should show dark mode text, got: {}",
        html
    );
    assert!(
        html.contains("🌙"),
        "Should show moon icon for dark mode, got: {}",
        html
    );
}

#[test]
fn test_sidebar_renders_running_state() {
    let mut dom = VirtualDom::new(TestSidebarRunning);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("Executing..."),
        "Should show executing text, got: {}",
        html
    );
    assert!(
        html.contains("animate-spin"),
        "Should have spinning animation, got: {}",
        html
    );
    assert!(
        html.contains("Running"),
        "Should show running status, got: {}",
        html
    );
    assert!(
        html.contains("bg-blue-500"),
        "Should have blue status indicator, got: {}",
        html
    );
}

#[test]
fn test_sidebar_has_proper_structure() {
    let mut dom = VirtualDom::new(TestSidebarIdle);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("flex flex-col"),
        "Should be flex column, got: {}",
        html
    );
    assert!(
        html.contains("lg:w-64"),
        "Should have fixed width on desktop, got: {}",
        html
    );
    assert!(
        html.contains("border-r"),
        "Should have right border, got: {}",
        html
    );
    assert!(
        html.contains("rounded-lg"),
        "Should have rounded elements, got: {}",
        html
    );
}

#[test]
fn test_sidebar_file_input_functionality() {
    let mut dom = VirtualDom::new(TestSidebarIdle);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("test.json"),
        "Should show current file value, got: {}",
        html
    );
    assert!(
        html.contains("Select workflow file..."),
        "Should have placeholder text, got: {}",
        html
    );
    assert!(
        html.contains("Browse Files"),
        "Should have browse button, got: {}",
        html
    );
    assert!(
        html.contains("📁"),
        "Should have folder icon, got: {}",
        html
    );
}
