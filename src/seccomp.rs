use crate::error::JarError;
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};

pub struct SeccompFilter;

impl SeccompFilter {
    pub fn apply_default_profile() -> Result<(), JarError> {
        // Default action: Deny unpermitted syscalls with EPERM (Operation Not Permitted)
        let mut ctx = ScmpFilterContext::new_filter(ScmpAction::Errno(libc::EPERM))
            .map_err(|e| JarError::Execution(format!("Failed to init seccomp context: {}", e)))?;

        // Essential Syscall Whitelist for core execution and Rust runtime
        let allowed_syscalls = [
            "read",
            "write",
            "openat",
            "close",
            "fstat",
            "lseek",
            "mmap",
            "mprotect",
            "munmap",
            "brk",
            "rt_sigaction",
            "rt_sigprocmask",
            "rt_sigreturn",
            "ioctl",
            "pread64",
            "pwrite64",
            "readv",
            "writev",
            "access",
            "pipe2",
            "select",
            "sched_yield",
            "dup",
            "dup2",
            "dup3",
            "fcntl",
            "getpid",
            "gettid",
            "getuid",
            "getgid",
            "geteuid",
            "getegid",
            "sigaltstack",
            "exit",
            "exit_group",
            "futex",
            "execve",
            "arch_prctl",
            "set_tid_address",
            "set_robust_list",
            "getrandom",
            "clock_gettime",
        ];

        for name in &allowed_syscalls {
            if let Ok(syscall) = ScmpSyscall::from_name(name) {
                let _ = ctx.add_arch(libseccomp::ScmpArch::Native);
                ctx.add_rule(ScmpAction::Allow, syscall).map_err(|e| {
                    JarError::Execution(format!("Failed to allow syscall {}: {}", name, e))
                })?;
            }
        }

        // Explicitly load BPF filter into the calling child process context
        ctx.load().map_err(|e| {
            JarError::Execution(format!("Failed to load seccomp BPF filter: {}", e))
        })?;

        Ok(())
    }
}
