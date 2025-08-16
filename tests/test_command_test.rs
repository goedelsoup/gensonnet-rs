use std::process::Command;

#[test]
fn test_test_command_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "test", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Run plugin tests"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("info"));
    assert!(stdout.contains("report"));
}

#[test]
fn test_test_list_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "test", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available test suites:"));
    assert!(stdout.contains("go-ast:builtin-tests"));
    assert!(stdout.contains("crd:builtin-tests"));
    assert!(stdout.contains("openapi:builtin-tests"));
}

#[test]
fn test_test_list_detailed_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "test", "list", "--detailed"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available test suites:"));
    assert!(stdout.contains("Description:"));
    assert!(stdout.contains("Type: Built-in"));
    assert!(stdout.contains("Test Cases:"));
}

#[test]
fn test_test_info_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "test", "info", "go-ast:builtin"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test Suite Information:"));
    assert!(stdout.contains("Name: go-ast:builtin"));
    assert!(stdout.contains("Description:"));
    assert!(stdout.contains("Test Cases:"));
    assert!(stdout.contains("Plugin ID:"));
    assert!(stdout.contains("Enabled Capabilities:"));
}

#[test]
fn test_test_run_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "test", "run", "--plugin-id", "go-ast:builtin"])
        .output()
        .expect("Failed to execute command");

    // The command should run but tests will fail since there's no actual test runner
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Running plugin tests..."));
    assert!(stdout.contains("Test Suite: go-ast:builtin-tests"));
    assert!(stdout.contains("Total Tests:"));
    assert!(stdout.contains("Passed:"));
    assert!(stdout.contains("Failed:"));
    assert!(stdout.contains("Total Time:"));
}
