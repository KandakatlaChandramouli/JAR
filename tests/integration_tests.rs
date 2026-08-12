use std::process::Command;

fn jar_bin() -> String {
    env!("CARGO_BIN_EXE_jar").to_string()
}

#[test]
fn test_run_valid_command() {
    let output = Command::new(jar_bin())
        .args(["run", "/bin/echo", "hello-jar"])
        .output()
        .expect("Failed to execute jar binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[jar] preparing execution"));
    assert!(stdout.contains("[jar] process started in isolated user/mount/PID namespaces"));
    assert!(stdout.contains("hello-jar"));
    assert!(stdout.contains("[jar] process exited: 0"));
}

#[test]
fn test_user_namespace_id_mapping() {
    let output = Command::new(jar_bin())
        .args(["run", "/usr/bin/id", "-u"])
        .output()
        .expect("Failed to execute jar binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0"));
}

#[test]
fn test_run_missing_executable() {
    let output = Command::new(jar_bin())
        .args(["run", "/bin/does-not-exist-jar-test"])
        .output()
        .expect("Failed to execute jar binary");

    assert!(!output.status.success());
}

#[test]
fn test_invalid_cli_args() {
    let output = Command::new(jar_bin())
        .arg("unknown-command")
        .output()
        .expect("Failed to execute jar binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Argument error: Unknown command"));
}

#[test]
fn test_nonzero_exit_propagation() {
    let output = Command::new(jar_bin())
        .args(["run", "/bin/sh", "-c", "exit 42"])
        .output()
        .expect("Failed to execute jar binary");

    assert_eq!(output.status.code(), Some(42));
}
