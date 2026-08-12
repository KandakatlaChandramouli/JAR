# JAR Architecture

## Purpose
JAR is a progressively extensible Linux process-isolation runtime. The goal is to build a minimal, correct, auditable sandbox from first principles, establishing clean boundaries before introducing advanced isolation primitives.

## v0.1 Architecture
The v0.1 release focuses strictly on the lifecycle abstractions:
- **CLI (`cli.rs`)**: Minimal argument parsing.
- **Sandbox (`sandbox.rs`)**: State management and validation. Defines the lifecycle (prepare -> validate -> run -> cleanup).
- **Process (`process.rs`)**: The execution primitive. Encapsulates host-level process spawning and standard I/O forwarding.

## Trust Boundaries and Security Model
**In v0.1, there is NO security boundary.** 
- The guest process shares the same PID namespace, mount namespace, and network namespace as the host.
- The guest process executes with the privileges of the user who invoked `jar`.
- Unrestricted filesystem access is permitted.

This version establishes a testable baseline to measure lifecycle overhead. It is an "execution runtime prototype", not a secure sandbox.

## Planned Isolation Layers
- **v0.2**: Filesystem isolation (chroot / pivot_root)
- **v0.3-v0.6**: Namespaces (PID, Mount, User, Network)
- **v0.7-v0.9**: Cgroups, seccomp, capability reduction
