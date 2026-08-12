use crate::cgroup::{CgroupManager, ResourceLimits};
use crate::error::JarError;
use crate::process::{ProcessExecutor, ProcessSpec};

pub struct SandboxConfig {
    pub executable: String,
    pub args: Vec<String>,
    pub rootfs: Option<String>,
    pub limits: Option<ResourceLimits>,
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

        // Initialize Cgroups manager if limits or default isolation requested
        let _cgroup = if let Some(ref limits) = self.config.limits {
            println!("[jar] applying cgroups v2 resource limits");
            let cg = CgroupManager::new("sandbox_exec")?;
            cg.apply_limits(limits)?;
            Some(cg)
        } else {
            None
        };

        let spec = ProcessSpec {
            executable: self.config.executable.clone(),
            args: self.config.args.clone(),
            rootfs: self.config.rootfs.clone(),
        };

        println!("[jar] process started in isolated user/mount/PID namespaces");

        let exit_code = ProcessExecutor::execute(&spec)?;

        println!("[jar] process exited: {}", exit_code);

        Ok(exit_code)
    }
}
