use crate::cgroup::{CgroupManager, ResourceLimits};
use crate::cli::RunOptions;
use crate::error::JarError;
use crate::overlay::OverlayManager;
use crate::process::{ProcessExecutor, ProcessSpec};

pub struct SandboxConfig {
    pub executable: String,
    pub args: Vec<String>,
    pub rootfs: Option<String>,
    pub limits: ResourceLimits,
    pub enable_seccomp: bool,
    pub drop_capabilities: bool,
}

impl From<RunOptions> for SandboxConfig {
    fn from(opts: RunOptions) -> Self {
        Self {
            executable: opts.executable,
            args: opts.args,
            rootfs: opts.rootfs,
            limits: opts.limits,
            enable_seccomp: opts.enable_seccomp,
            drop_capabilities: opts.drop_capabilities,
        }
    }
}

pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Sandbox { config }
    }

    fn validate(&self) -> Result<(), JarError> {
        if self.config.executable.is_empty() {
            return Err(JarError::Validation(
                "Executable path cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    pub fn run(&self) -> Result<i32, JarError> {
        println!("[jar] preparing execution");
        self.validate()?;

        println!("[jar] executable: {}", self.config.executable);

        let _cgroup = if self.config.limits.memory_max_bytes.is_some()
            || self.config.limits.pids_max.is_some()
        {
            println!("[jar] applying cgroups v2 resource limits");
            let cg = CgroupManager::new("sandbox_exec")?;
            cg.apply_limits(&self.config.limits)?;
            Some(cg)
        } else {
            None
        };

        let overlay = if let Some(ref lower_path) = self.config.rootfs {
            println!("[jar] setting up OverlayFS copy-on-write filesystem layer");
            Some(OverlayManager::new("sandbox_exec", lower_path)?)
        } else {
            None
        };

        let spec = ProcessSpec {
            executable: self.config.executable.clone(),
            args: self.config.args.clone(),
            rootfs: self.config.rootfs.clone(),
            overlay: overlay.clone(),
            enable_seccomp: self.config.enable_seccomp,
            drop_capabilities: self.config.drop_capabilities,
        };

        println!("[jar] process started in isolated user/mount/PID namespaces");

        let exit_code = ProcessExecutor::execute(&spec);

        if let Some(ref mgr) = overlay {
            mgr.cleanup();
        }

        exit_code
    }
}
