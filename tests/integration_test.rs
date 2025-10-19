use std::fs;
use std::process::Command;
use std::sync::Mutex;

// Use a mutex to ensure tests run sequentially and don't interfere
static TEST_MUTEX: Mutex<()> = Mutex::new(());

// Test helper functions
fn create_test_workflow(content: &str) {
    fs::write("test_workflow.json", content).unwrap();
}

fn run_workflow_app() -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--quiet", "--bin", "cli", "--", "test_workflow.json"])
        .output()
        .unwrap()
}

fn cleanup_workflow() {
    let _ = fs::remove_file("test_workflow.json");
}

fn load_fixture(fixture_name: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{}.json", fixture_name)).unwrap()
}

// Helper to run tests with proper isolation
fn run_test<F>(test_fn: F)
where
    F: FnOnce(),
{
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    test_fn();
}

#[test]
fn test_simple_echo_workflow() {
    run_test(|| {
        // Create workflow with simple echo command using dataflow-rs format
        create_test_workflow(&load_fixture("valid_workflow"));

        let output = run_workflow_app();

        // Verify exit code is 0 (success)
        assert!(
            output.status.success(),
            "Expected successful execution, got exit code: {:?}",
            output.status.code()
        );

        // Verify output contains expected text
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello from workflow!"),
            "Expected 'Hello from workflow!' in output, got: {}",
            stdout
        );
        assert!(
            stdout.contains("Workflow execution complete"),
            "Expected completion message, got: {}",
            stdout
        );
        assert!(
            stdout.contains("hello_step"),
            "Expected task ID in audit trail, got: {}",
            stdout
        );

        cleanup_workflow();
    });
}

#[test]
fn test_multi_step_workflow_all_execute() {
    run_test(|| {
        // Test that ALL steps execute (not just the first one)
        create_test_workflow(&load_fixture("multi_step_workflow"));

        let output = run_workflow_app();

        assert!(output.status.success(), "Expected successful execution");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Both steps should execute
        assert!(
            stdout.contains("First step executed"),
            "Expected first step output, got: {}",
            stdout
        );
        assert!(
            stdout.contains("Second step executed"),
            "Expected second step output, got: {}",
            stdout
        );

        // Check audit trail shows both tasks
        assert!(
            stdout.contains("first_step"),
            "Expected first_step in audit trail"
        );
        assert!(
            stdout.contains("second_step"),
            "Expected second_step in audit trail"
        );

        // Should show 2 tasks in the audit trail
        assert!(
            stdout.contains("Number of tasks: 2"),
            "Expected 2 tasks message"
        );

        cleanup_workflow();
    });
}

#[test]
fn test_command_with_multiple_args() {
    run_test(|| {
        // Test echo with multiple arguments using dataflow-rs format
        let workflow_json = r#"{
          "id": "multi_arg_test",
          "name": "Multi-Arg Test",
          "tasks": [
            {
              "id": "multi_arg_step",
              "name": "Multi Arg Step",
              "function": {
                "name": "cli_command",
                "input": {
                  "command": "echo",
                  "args": ["Hello", "World", "from", "Rust!"]
                }
              }
            }
          ]
        }"#;

        create_test_workflow(workflow_json);

        let output = run_workflow_app();

        assert!(output.status.success(), "Expected successful execution");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello World from Rust!"),
            "Expected multi-arg output, got: {}",
            stdout
        );

        cleanup_workflow();
    });
}

#[test]
fn test_different_cli_commands() {
    run_test(|| {
        // Test with different commands
        let commands: Vec<(&str, &str)> = vec![("pwd", "pwd_step"), ("date", "date_step")];

        for (cmd, step_id) in commands {
            let workflow_json = format!(
                r#"{{
              "id": "{}_test",
              "name": "{} Test",
              "tasks": [
                {{
                  "id": "{}",
                  "name": "{} Step",
                  "function": {{
                    "name": "cli_command",
                    "input": {{
                      "command": "{}",
                      "args": []
                    }}
                  }}
                }}
              ]
            }}"#,
                cmd, cmd, step_id, cmd, cmd
            );

            create_test_workflow(&workflow_json);

            let output = run_workflow_app();

            assert!(
                output.status.success(),
                "Expected successful execution for {}",
                cmd
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!stdout.is_empty(), "Expected non-empty output for {}", cmd);
            assert!(
                stdout.contains("Workflow execution complete"),
                "Expected completion message for {}",
                cmd
            );

            cleanup_workflow();
        }
    });
}

#[test]
fn test_invalid_command_fails() {
    run_test(|| {
        // Test with non-existent command
        create_test_workflow(&load_fixture("invalid_command"));

        let output = run_workflow_app();

        // Should fail with non-zero exit code
        assert!(
            !output.status.success(),
            "Expected failure for invalid command, got success"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should contain error message about command failure
        let combined = format!("{}{}", stderr, stdout);
        assert!(
            combined.contains("Failed to execute command")
                || combined.contains("Workflow execution failed")
                || combined.contains("nonexistent_command")
                || combined.contains("not found")
                || combined.contains("No such file"),
            "Expected error message about failed command, stderr: {}, stdout: {}",
            stderr,
            stdout
        );

        cleanup_workflow();
    });
}

#[test]
fn test_malformed_json() {
    run_test(|| {
        // Test with invalid JSON
        create_test_workflow(&load_fixture("invalid_json"));

        let output = run_workflow_app();

        // Should fail with non-zero exit code
        assert!(
            !output.status.success(),
            "Expected failure for malformed JSON, got success"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should contain JSON parsing error
        let combined = format!("{}{}", stderr, stdout);
        assert!(
            combined.contains("Failed to parse workflow")
                || combined.contains("JSON")
                || combined.contains("parse")
                || combined.contains("expected")
                || combined.contains("EOF"),
            "Expected JSON error message, stderr: {}, stdout: {}",
            stderr,
            stdout
        );

        cleanup_workflow();
    });
}

#[test]
fn test_missing_workflow_file() {
    run_test(|| {
        // Test with a non-existent file
        let output = Command::new("cargo")
            .args([
                "run",
                "--quiet",
                "--bin",
                "cli",
                "--",
                "nonexistent_workflow.json",
            ])
            .output()
            .unwrap();

        // Should fail with non-zero exit code
        assert!(
            !output.status.success(),
            "Expected failure for missing file, got success"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should contain file not found error
        let combined = format!("{}{}", stderr, stdout);
        assert!(
            combined.contains("Failed to read workflow file")
                || combined.contains("No such file")
                || combined.contains("not found")
                || combined.contains("nonexistent_workflow.json"),
            "Expected file not found error, stderr: {}, stdout: {}",
            stderr,
            stdout
        );
    });
}

#[test]
fn test_empty_workflow_tasks() {
    run_test(|| {
        // Test with empty tasks array
        let workflow_json = r#"{
          "id": "empty_workflow",
          "name": "Empty Workflow",
          "tasks": []
        }"#;

        create_test_workflow(workflow_json);

        let output = run_workflow_app();

        // Should succeed even with no tasks
        assert!(
            output.status.success(),
            "Expected successful execution with empty tasks"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Number of tasks: 0"),
            "Expected 0 tasks message, got: {}",
            stdout
        );
        assert!(
            stdout.contains("Workflow execution complete"),
            "Expected completion message"
        );

        cleanup_workflow();
    });
}

#[test]
fn test_workflow_audit_trail() {
    run_test(|| {
        // Test that audit trail is properly generated
        create_test_workflow(&load_fixture("multi_step_workflow"));

        let output = run_workflow_app();

        assert!(output.status.success(), "Expected successful execution");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for audit trail section
        assert!(
            stdout.contains("Audit Trail"),
            "Expected 'Audit Trail' section"
        );

        // Check that both tasks appear in audit trail
        assert!(
            stdout.contains("first_step"),
            "Expected first_step in audit trail"
        );
        assert!(
            stdout.contains("second_step"),
            "Expected second_step in audit trail"
        );

        // Check for status codes
        assert!(
            stdout.contains("Status: 200"),
            "Expected success status in audit trail"
        );

        // Check for timestamps
        assert!(
            stdout.contains("Timestamp:"),
            "Expected timestamp in audit trail"
        );

        // Check for changes tracking
        assert!(
            stdout.contains("Changes:"),
            "Expected changes tracking in audit trail"
        );

        cleanup_workflow();
    });
}

#[test]
fn test_workflow_context() {
    run_test(|| {
        // Test that context is properly maintained
        create_test_workflow(&load_fixture("valid_workflow"));

        let output = run_workflow_app();

        assert!(output.status.success(), "Expected successful execution");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for final context section
        assert!(
            stdout.contains("Final Context"),
            "Expected 'Final Context' section"
        );

        // Context should contain CLI output
        assert!(
            stdout.contains("cli_output"),
            "Expected cli_output in context"
        );
        assert!(
            stdout.contains("stdout"),
            "Expected stdout field in context"
        );
        assert!(
            stdout.contains("exit_code"),
            "Expected exit_code field in context"
        );

        cleanup_workflow();
    });
}

#[test]
fn test_per_task_logs_populated() {
    run_test(|| {
        create_test_workflow(&load_fixture("multi_step_workflow"));
        
        // Execute via library (not CLI binary)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            simple_workflow_app::execute_workflow("test_workflow.json", None).await
        }).unwrap();
        
        // THIS IS THE CORE BUG FIX - per_task_logs should be populated!
        assert!(!result.per_task_logs.is_empty(), "per_task_logs should not be empty");
        assert!(result.per_task_logs.contains_key("first_step"));
        assert!(result.per_task_logs.contains_key("second_step"));
        
        cleanup_workflow();
    });
}
