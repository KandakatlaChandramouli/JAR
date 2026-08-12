# JAR v0.1 Security Limitations

**DO NOT DEPLOY IN UNTRUSTED ENVIRONMENTS.**

## What JAR Isolates in v0.1
- Nothing.

## What JAR does NOT isolate
- **Filesystem:** The child process has full access to the host filesystem.
- **Process visibility:** The child can see and signal host PIDs.
- **Network:** The child binds to host network interfaces.
- **Resources:** CPU and memory usage are unbounded.
