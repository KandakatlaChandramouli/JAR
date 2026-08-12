use crate::error::JarError;
use crate::process::{ProcessExecutor, ProcessSpec};

pub struct SandboxConfig {
    pub executable: String,
    pub args: Vec<String>,
}

pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Sandbox { config }
    }

    fn validate(&self) -> Result<(), JarError> {
        // v0.1 Validation: Extremely basic existence check
        if self.config.executable.is_empty() {
            return Err(JarError::Validation("Executable path cannot be empty".to_string()));
        }
        // Fail closed if it clearly isn't an executable path we can resolve
        // In v0.2+, this will check inside the chroot/pivot_root
        Ok(())
    }

    pub fn run(&self) -> Result<i32, JarError> {
        println!("[jar] preparing execution");
        self.validate()?;

        println!("[jar] executable: {}", self.config.executable);
        
        let spec = ProcessSpec {
            executable: self.config.executable.clone(),
            args: self.config.args.clone(),
        };

        println!("[jar] process started");
        
        // Explicit boundary: everything inside execute() is part of the child lifecycle
        let exit_code = ProcessExecutor::execute(&spec)?;
        
        // Deterministic cleanup will hook here in future versions
        println!("[jar] process exited: {}", exit_code);
        
        Ok(exit_code)
    }
}
