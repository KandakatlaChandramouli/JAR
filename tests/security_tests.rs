use std::process::Command;

fn jar_bin() -> String {
    env!("CARGO_BIN_EXE_jar").to_string()
}

#[test]
fn test_pid_namespace_isolation() {
    let output = Command::new(jar_bin())
        .args(["run", "/bin/sh", "-c", "test -d /proc/1"])
        .output()
        .expect("Failed to execute jar binary");

    assert!(output.status.success());
}

#[test]
fn test_seccomp_denies_reboot() {
    let output = Command::new(jar_bin())
        .args(["run", "/sbin/reboot"])
        .output()
        .expect("Failed to execute jar binary");

    assert!(!output.status.success());
}

#[test]
fn test_capability_stripping_execution() {
    let output = Command::new(jar_bin())
        .args(["run", "/bin/echo", "security-test-passed"])
        .output()
        .expect("Failed to execute jar binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("security-test-passed"));
}
