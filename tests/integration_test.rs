use std::process::Command;
use std::fs;
use std::sync::Mutex;

// Use a mutex to ensure tests run sequentially and don't interfere
static TEST_MUTEX: Mutex<()> = Mutex::new(());

// Test helper functions
fn create_test_workflow(content: &str) {
    // Create workflow.json in the project root (where the binary expects it)
    fs::write("workflow.json", content).unwrap();
}

fn run_workflow_app() -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--quiet"])
        .output()
        .unwrap()
}

fn cleanup_workflow() {
    let _ = fs::remove_file("workflow.json");
}

fn load_fixture(fixture_name: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{}.json", fixture_name)).unwrap()
}

// Helper to run tests with proper isolation
fn run_test<F>(test_fn: F) where F: FnOnce() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    test_fn();
}

#[test]
fn test_simple_echo_workflow() {
    run_test(|| {
        // Create workflow with simple echo command
        create_test_workflow(&load_fixture("valid_workflow"));
        
        let output = run_workflow_app();
        
        // Verify exit code is 0 (success)
        assert!(output.status.success(), "Expected successful execution, got exit code: {:?}", output.status.code());
        
        // Verify output contains expected text
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Hello from workflow!"), "Expected 'Hello from workflow!' in output, got: {}", stdout);
        assert!(stdout.contains("Step 'hello_step' completed successfully"), "Expected success message, got: {}", stdout);
        
        cleanup_workflow();
    });
}

#[test]
fn test_command_with_multiple_args() {
    run_test(|| {
        // Test echo with multiple arguments
        let workflow_json = r#"{
          "steps": [
            {
              "id": "multi_arg_step",
              "type": "cli",
              "command": "echo",
              "args": ["Hello", "World", "from", "Rust!"]
            }
          ]
        }"#;
        
        create_test_workflow(workflow_json);
        
        let output = run_workflow_app();
        
        assert!(output.status.success(), "Expected successful execution");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Hello World from Rust!"), "Expected multi-arg output, got: {}", stdout);
        
        cleanup_workflow();
    });
}

#[test]
fn test_multiple_steps_only_first_executes() {
    run_test(|| {
        // Test that only the first step executes even with multiple steps
        create_test_workflow(&load_fixture("multi_step_workflow"));
        
        let output = run_workflow_app();
        
        assert!(output.status.success(), "Expected successful execution");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("First step executed"), "Expected first step output, got: {}", stdout);
        assert!(!stdout.contains("Second step should not run"), "Second step should not have executed, got: {}", stdout);
        assert!(stdout.contains("Loaded workflow with 2 steps"), "Expected 2 steps loaded message");
        
        cleanup_workflow();
    });
}

#[test]
fn test_different_cli_commands() {
    run_test(|| {
        // Test with different commands
        let commands: Vec<(&str, Vec<&str>, &str)> = vec![
            ("pwd", vec![], "current directory path"),
            ("date", vec![], "date output"),
        ];
        
        for (test_name, cmd_args, _expected_content) in commands {
            let workflow_json = format!(r#"{{
              "steps": [
                {{
                  "id": "{}_step",
                  "type": "cli",
                  "command": "{}",
                  "args": {:?}
                }}
              ]
            }}"#, test_name, test_name, cmd_args);
            
            create_test_workflow(&workflow_json);
            
            let output = run_workflow_app();
            
            assert!(output.status.success(), "Expected successful execution for {}", test_name);
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!stdout.is_empty(), "Expected non-empty output for {}", test_name);
            
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
        assert!(!output.status.success(), "Expected failure for invalid command, got success");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should contain error message about command failure
        assert!(stderr.contains("Command failed") || stdout.contains("Command failed") || 
                stderr.contains("not found") || stdout.contains("not found") ||
                stderr.contains("No such file") || stdout.contains("No such file") ||
                stderr.contains("Os { code: 2") || stdout.contains("Os { code: 2"), 
                "Expected error message, stderr: {}, stdout: {}", stderr, stdout);
        
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
        assert!(!output.status.success(), "Expected failure for malformed JSON, got success");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should contain JSON parsing error
        assert!(stderr.contains("JSON") || stdout.contains("JSON") || 
                stderr.contains("parse") || stdout.contains("parse") ||
                stderr.contains("invalid") || stdout.contains("invalid") ||
                stderr.contains("trailing characters") || stdout.contains("trailing characters") ||
                stderr.contains("Error(") || stdout.contains("Error("), 
                "Expected JSON error message, stderr: {}, stdout: {}", stderr, stdout);
        
        cleanup_workflow();
    });
}

#[test]
fn test_missing_workflow_file() {
    run_test(|| {
        // Ensure workflow.json doesn't exist
        cleanup_workflow();
        
        let output = run_workflow_app();
        
        // Should fail with non-zero exit code
        assert!(!output.status.success(), "Expected failure for missing file, got success");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should contain file not found error
        assert!(stderr.contains("No such file") || stdout.contains("No such file") ||
                stderr.contains("not found") || stdout.contains("not found") ||
                stderr.contains("workflow.json") || stdout.contains("workflow.json"), 
                "Expected file not found error, stderr: {}, stdout: {}", stderr, stdout);
        
        // Restore a valid workflow for other tests
        create_test_workflow(&load_fixture("valid_workflow"));
    });
}

#[test]
fn test_empty_workflow_steps() {
    run_test(|| {
        // Test with empty steps array
        let workflow_json = r#"{
          "steps": []
        }"#;
        
        create_test_workflow(workflow_json);
        
        let output = run_workflow_app();
        
        assert!(output.status.success(), "Expected successful execution with empty steps");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("No steps found in workflow"), "Expected no steps message, got: {}", stdout);
        assert!(stdout.contains("Loaded workflow with 0 steps"), "Expected 0 steps loaded message");
        
        cleanup_workflow();
    });
}