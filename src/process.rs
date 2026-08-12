use crate::capabilities::CapabilityManager;
use crate::error::JarError;
use crate::overlay::OverlayManager;
use crate::seccomp::SeccompFilter;
use nix::mount::{mount, MntFlags, MsFlags};
use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{chdir, chroot, getgid, getuid, pivot_root, Pid};
use std::ffi::CString;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub struct ProcessSpec {
    pub executable: String,
    pub args: Vec<String>,
    pub rootfs: Option<String>,
    pub overlay: Option<OverlayManager>,
    pub enable_seccomp: bool,
    pub drop_capabilities: bool,
}

pub struct ProcessExecutor;

impl ProcessExecutor {
    pub fn execute(spec: &ProcessSpec) -> Result<i32, JarError> {
        const STACK_SIZE: usize = 1024 * 1024;
        let mut stack = vec![0u8; STACK_SIZE];

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
        let _ = mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        );

        let effective_rootfs = if let Some(ref overlay) = spec.overlay {
            if overlay.mount_overlay()? {
                Some(overlay.merged_dir.to_string_lossy().to_string())
            } else {
                spec.rootfs.clone()
            }
        } else {
            spec.rootfs.clone()
        };

        if let Some(ref rootfs) = effective_rootfs {
            Self::setup_rootfs(rootfs)?;
        } else {
            let _ = mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
                None::<&str>,
            );
        }

        if spec.enable_seccomp {
            SeccompFilter::apply_default_profile()?;
        }

        if spec.drop_capabilities {
            CapabilityManager::drop_all_capabilities()?;
        }

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

    fn setup_rootfs(new_root: &str) -> Result<(), JarError> {
        let new_root_path = Path::new(new_root);

        let _ = mount(
            Some(new_root_path),
            new_root_path,
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        );

        let old_root = new_root_path.join(".old_root");
        create_dir_all(&old_root)?;

        if pivot_root(new_root_path, &old_root).is_ok() {
            let _ = chdir("/");
            let _ = create_dir_all("/proc");
            let _ = mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
                None::<&str>,
            );
            let _ = nix::mount::umount2("/.old_root", MntFlags::MNT_DETACH);
            let _ = std::fs::remove_dir("/.old_root");
        } else {
            let _ = chroot(new_root_path);
            let _ = chdir("/");
        }

        Ok(())
    }
}
