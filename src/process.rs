use crate::error::JarError;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct ProcessSpec {
    pub executable: String,
    pub args: Vec<String>,
}

pub struct ProcessExecutor;

impl ProcessExecutor {
    pub fn execute(spec: &ProcessSpec) -> Result<i32, JarError> {
        // v0.1 Minimal Execution: Spawn process directly on host
        let mut child = Command::new(&spec.executable)
            .args(&spec.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| JarError::Execution(format!("Failed to spawn child process: {}", e)))?;

        let status = child
            .wait()
            .map_err(|e| JarError::Execution(format!("Failed to wait for child process: {}", e)))?;

        Ok(status.code().unwrap_or(128)) // 128 as fallback for signal termination
    }
}
