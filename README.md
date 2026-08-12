# JAR - Linux Process-Isolation Container Runtime

**JAR (v1.0.0)** is a lightweight, extensible Linux process isolation container runtime written in pure Rust. It leverages Linux kernel primitives—namespaces, cgroups v2, Seccomp BPF filters, POSIX capabilities, and OverlayFS—to construct fully isolated process sandboxes without daemon overhead.

## Key Features
- **Multi-Namespace Isolation**: Enforces separate User (`CLONE_NEWUSER`), Mount (`CLONE_NEWNS`), and PID (`CLONE_NEWPID`) namespaces.
- **Rootless Mapping**: Maps host unprivileged UIDs to container `root` (UID 0) inside user namespaces.
- **Copy-on-Write Filesystems**: Mounts ephemeral **OverlayFS** scratchpads (`upper`, `work`, `merged`) over base rootfs targets or extracted OCI image archives.
- **OCI/Docker Image Support**: Unpacks `.tar` and `.tar.gz`/`.tgz` exported container image tarballs directly.
- **Cgroups v2 Resource Constraints**: Restricts maximum memory usage (`memory.max`) and process tree counts (`pids.max`).
- **Syscall Filtering**: Applies strict default `EPERM` Seccomp BPF execution whitelists.
- **Privilege Boundary Dropping**: Strips Effective, Permitted, Inheritable, and Ambient POSIX capabilities prior to process execution.

## Installation & Building

```bash
cargo build --release
