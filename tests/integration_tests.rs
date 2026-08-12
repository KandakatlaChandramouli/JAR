use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
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
fn test_run_with_memory_and_pids_flags() {
    let output = Command::new(jar_bin())
        .args([
            "run",
            "--memory",
            "536870912",
            "--pids",
            "100",
            "/bin/echo",
            "cgroup-limits-test",
        ])
        .output()
        .expect("Failed to execute jar binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[jar] applying cgroups v2 resource limits"));
    assert!(stdout.contains("cgroup-limits-test"));
    assert!(stdout.contains("[jar] process exited: 0"));
}

#[test]
fn test_overlayfs_copy_on_write_rootfs() {
    let test_rootfs = "/tmp/jar_test_rootfs";
    let _ = remove_dir_all(test_rootfs);
    create_dir_all(format!("{}/bin", test_rootfs)).expect("Failed to create test rootfs bin");

    // Copy host /bin/echo into dummy rootfs
    std::fs::copy("/bin/echo", format!("{}/bin/echo", test_rootfs))
        .expect("Failed to copy echo binary to test rootfs");

    let output = Command::new(jar_bin())
        .args([
            "run",
            "--rootfs",
            test_rootfs,
            "/bin/echo",
            "overlay-cow-test",
        ])
        .output()
        .expect("Failed to execute jar binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _ = remove_dir_all(test_rootfs);

    assert!(output.status.success());
    assert!(stdout.contains("[jar] setting up OverlayFS copy-on-write filesystem layer"));
    assert!(stdout.contains("overlay-cow-test"));
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
