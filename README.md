# JAR — Just Another Runtime

**JAR** is a research-grade Linux process-isolation runtime built from first principles.

## Project Overview
The immediate objective is to build a minimal, correct, auditable sandbox that can execute an untrusted Linux process inside a controlled environment. 

JAR explicitly avoids wrapping existing container runtimes, focusing instead on direct implementation of Linux isolation primitives.

## Current Capabilities (v0.1)
- Deterministic process execution
- Direct stdio forwarding
- Exit-status propagation
- Core lifecycle abstractions

*Note: v0.1 provides NO security isolation. See `docs/security_limitations.md`.*

## Build and Run
```bash
cargo build --release
./target/release/jar run /bin/echo "Hello, JAR"
quit
