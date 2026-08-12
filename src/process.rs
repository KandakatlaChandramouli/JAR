use crate::error::JarError;
use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{getgid, getuid, Pid};
use std::ffi::CString;
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub struct ProcessSpec {
    pub executable: String,
    pub args: Vec<String>,
}

pub struct ProcessExecutor;

impl ProcessExecutor {
    pub fn execute(spec: &ProcessSpec) -> Result<i32, JarError> {
        // Allocate a 1MB stack for the child process execution
        const STACK_SIZE: usize = 1024 * 1024;
        let mut stack = vec![0u8; STACK_SIZE];

        // Combine CLONE_NEWUSER and CLONE_NEWNS flags for unprivileged namespace creation
        let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS;

        // Child process closure
        let child_fn = Box::new(|| -> isize {
            match Self::child_entrypoint(spec) {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("[jar child error]: {}", e);
                    1
                }
            }
        });

        // Spawn child process into new user and mount namespaces
        let child_pid =
            unsafe { clone(child_fn, &mut stack, flags, Some(Signal::SIGCHLD as i32))? };

        // Write UID and GID mappings from host parent before child executes execve
        Self::setup_user_mappings(child_pid)?;

        // Wait for child process termination and return exit status
        match waitpid(child_pid, None)? {
            WaitStatus::Exited(_, status) => Ok(status),
            WaitStatus::Signaled(_, sig, _) => Ok(128 + sig as i32),
            _ => Ok(1),
        }
    }

    fn setup_user_mappings(pid: Pid) -> Result<(), JarError> {
        let uid = getuid();
        let gid = getgid();

        // Deny setgroups to allow unprivileged GID mapping
        if let Ok(mut setgroups) = File::create(format!("/proc/{}/setgroups", pid)) {
            let _ = setgroups.write_all(b"deny");
        }

        // Map container root (0) to host user UID
        let mut uid_map = File::create(format!("/proc/{}/uid_map", pid))?;
        uid_map.write_all(format!("0 {} 1\n", uid).as_bytes())?;

        // Map container root (0) to host group GID
        let mut gid_map = File::create(format!("/proc/{}/gid_map", pid))?;
        gid_map.write_all(format!("0 {} 1\n", gid).as_bytes())?;

        Ok(())
    }

    fn child_entrypoint(spec: &ProcessSpec) -> Result<(), JarError> {
        // Convert executable string to CString for execvp call
        let c_executable = CString::new(spec.executable.clone())
            .map_err(|e| JarError::Execution(format!("Invalid path: {}", e)))?;

        // Prepare arguments as CStrings
        let mut c_args = Vec::new();
        c_args.push(c_executable.clone());
        for arg in &spec.args {
            c_args.push(
                CString::new(arg.clone())
                    .map_err(|e| JarError::Execution(format!("Invalid arg: {}", e)))?,
            );
        }

        // Execute binary inside isolated namespace
        nix::unistd::execvp(&c_executable, &c_args)?;

        Ok(())
    }
}
