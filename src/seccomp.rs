use crate::error::JarError;
use libseccomp::*;

pub struct SeccompFilter;

impl SeccompFilter {
    pub fn apply_default_profile() -> Result<(), JarError> {
        let mut filter = ScmpFilterContext::new_filter(ScmpAction::Errno(1))
            .map_err(|e| JarError::Execution(format!("Failed to create seccomp filter: {}", e)))?;

        let allowed_syscalls = [
            "read", "write", "open", "openat", "close", "stat", "fstat", "lstat",
            "newfstatat", "poll", "lseek", "mmap", "mprotect", "munmap", "brk",
            "rt_sigaction", "rt_sigprocmask", "rt_sigreturn", "ioctl", "pread64",
            "pwrite64", "readv", "writev", "access", "pipe", "pipe2", "select",
            "sched_yield", "mremap", "msync", "mincore", "madvise", "dup", "dup2",
            "dup3", "nanosleep", "clock_nanosleep", "getitimer", "setitimer",
            "alarm", "getpid", "getppid", "getuid", "geteuid", "getgid", "getegid",
            "getresuid", "getresgid", "clone", "clone3", "fork", "vfork", "execve",
            "execveat", "exit", "exit_group", "wait4", "waitid", "kill", "uname",
            "fcntl", "flock", "fsync", "fdatasync", "truncate", "ftruncate",
            "getdents", "getdents64", "getcwd", "chdir", "fchdir", "rename",
            "mkdir", "rmdir", "creat", "link", "unlink", "symlink", "readlink",
            "readlinkat", "chmod", "fchmod", "chown", "fchown", "lchown",
            "umask", "gettimeofday", "getrlimit", "getrusage", "sysinfo",
            "times", "ptrace", "getuid", "getgid", "geteuid", "getegid",
            "set_tid_address", "set_robust_list", "get_robust_list", "epoll_create",
            "epoll_create1", "epoll_ctl", "epoll_wait", "epoll_pwait", "eventfd",
            "eventfd2", "statx", "arch_prctl", "futex", "sched_getaffinity",
            "sched_setaffinity", "getrandom", "prlimit64", "close_range",
        ];

        for sys in allowed_syscalls {
            if let Ok(syscall_num) = ScmpSyscall::from_name(sys) {
                let _ = filter.add_rule(ScmpAction::Allow, syscall_num);
            }
        }

        filter
            .load()
            .map_err(|e| JarError::Execution(format!("Failed to load seccomp filter: {}", e)))?;

        Ok(())
    }
}
