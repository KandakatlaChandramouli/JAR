use std::fs::{create_dir_all, remove_dir_all, File};
use std::process::Command;
use tar::Builder;

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
fn test_oci_image_tarball_extraction() {
    let archive_path = "/tmp/mock_image.tar";
    let _ = std::fs::remove_file(archive_path);

    let src_dir = "/tmp/mock_image_src";
    let _ = remove_dir_all(src_dir);
    create_dir_all(format!("{}/bin", src_dir)).expect("Failed to create mock src bin");

    std::fs::copy("/bin/echo", format!("{}/bin/echo", src_dir))
        .expect("Failed to copy echo binary to mock src");

    let file = File::create(archive_path).expect("Failed to create mock image tar file");
    let mut a = Builder::new(file);

    // Append contents directly to root of tarball
    a.append_dir_all(".", src_dir)
        .expect("Failed to build mock tarball");
    a.finish().expect("Failed to finalize tarball");

    let output = Command::new(jar_bin())
        .args([
            "run",
            "--image",
            archive_path,
            "/bin/echo",
            "oci-image-test",
        ])
        .output()
        .expect("Failed to execute jar binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _ = std::fs::remove_file(archive_path);
    let _ = remove_dir_all(src_dir);

    assert!(stdout.contains("[jar] unpacking OCI container image layer into rootfs cache"));
    assert!(stdout.contains("[jar] setting up OverlayFS copy-on-write filesystem layer"));
}

#[test]
fn test_overlayfs_copy_on_write_rootfs() {
    let test_rootfs = "/tmp/jar_test_rootfs";
    let _ = remove_dir_all(test_rootfs);
    create_dir_all(format!("{}/bin", test_rootfs)).expect("Failed to create test rootfs bin");
    create_dir_all(format!("{}/lib", test_rootfs)).expect("Failed to create test rootfs lib");
    create_dir_all(format!("{}/lib64", test_rootfs)).expect("Failed to create test rootfs lib64");

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

    assert!(stdout.contains("[jar] setting up OverlayFS copy-on-write filesystem layer"));
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
