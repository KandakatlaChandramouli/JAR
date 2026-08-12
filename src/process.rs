use crate::error::JarError;
use nix::mount::{mount, MntFlags, MsFlags};
use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{chdir, getgid, getuid, pivot_root, Pid};
use std::ffi::CString;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub struct ProcessSpec {
    pub executable: String,
    pub args: Vec<String>,
    pub rootfs: Option<String>,
}

pub struct ProcessExecutor;

impl ProcessExecutor {
    pub fn execute(spec: &ProcessSpec) -> Result<i32, JarError> {
        const STACK_SIZE: usize = 1024 * 1024;
        let mut stack = vec![0u8; STACK_SIZE];

        // Combine User, Mount, and PID namespace flags
        let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID;

        let child_fn = Box::new(|| -> isize {
            match Self::child_entrypoint(spec) {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("[jar child error]: {}", e);
                    1
                }
            }
        });

        let child_pid =
            unsafe { clone(child_fn, &mut stack, flags, Some(Signal::SIGCHLD as i32))? };

        Self::setup_user_mappings(child_pid)?;

        match waitpid(child_pid, None)? {
            WaitStatus::Exited(_, status) => Ok(status),
            WaitStatus::Signaled(_, sig, _) => Ok(128 + sig as i32),
            _ => Ok(1),
        }
    }

    fn setup_user_mappings(pid: Pid) -> Result<(), JarError> {
        let uid = getuid();
        let gid = getgid();

        if let Ok(mut setgroups) = File::create(format!("/proc/{}/setgroups", pid)) {
            let _ = setgroups.write_all(b"deny");
        }

        let mut uid_map = File::create(format!("/proc/{}/uid_map", pid))?;
        uid_map.write_all(format!("0 {} 1\n", uid).as_bytes())?;

        let mut gid_map = File::create(format!("/proc/{}/gid_map", pid))?;
        gid_map.write_all(format!("0 {} 1\n", gid).as_bytes())?;

        Ok(())
    }

    fn child_entrypoint(spec: &ProcessSpec) -> Result<(), JarError> {
        // 1. Make mount propagation private so changes don't leak to host
        mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        )?;

        // 2. Perform pivot_root if custom rootfs provided, or setup fresh /proc
        if let Some(ref rootfs) = spec.rootfs {
            Self::setup_pivot_root(rootfs)?;
        } else {
            // Remount /proc in place for isolated PID space when running against host root
            let _ = mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
                None::<&str>,
            );
        }

        // 3. Execute target executable
        let c_executable = CString::new(spec.executable.clone())
            .map_err(|e| JarError::Execution(format!("Invalid path: {}", e)))?;

        let mut c_args = Vec::new();
        c_args.push(c_executable.clone());
        for arg in &spec.args {
            c_args.push(
                CString::new(arg.clone())
                    .map_err(|e| JarError::Execution(format!("Invalid arg: {}", e)))?,
            );
        }

        nix::unistd::execvp(&c_executable, &c_args)?;

        Ok(())
    }

    fn setup_pivot_root(new_root: &str) -> Result<(), JarError> {
        let new_root = Path::new(new_root);
        let old_root = new_root.join(".old_root");

        // Bind mount new_root to itself to satisfy pivot_root requirement
        mount(
            Some(new_root),
            new_root,
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        )?;

        create_dir_all(&old_root)?;

        pivot_root(new_root, &old_root)?;
        chdir("/")?;

        // Mount new /proc inside isolated filesystem
        create_dir_all("/proc")?;
        mount(
            Some("proc"),
            "/proc",
            Some("proc"),
            MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
            None::<&str>,
        )?;

        // Unmount old root and clean up
        nix::mount::umount2("/.old_root", MntFlags::MNT_DETACH)?;
        std::fs::remove_dir("/.old_root")?;

        Ok(())
    }
}
