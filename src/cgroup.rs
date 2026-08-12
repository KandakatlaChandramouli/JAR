use crate::error::JarError;
use nix::unistd::Pid;
use std::fs::{create_dir_all, remove_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResourceLimits {
    pub memory_max_bytes: Option<u64>,
    pub cpu_max_quota_us: Option<u64>,
    pub cpu_max_period_us: Option<u64>,
    pub pids_max: Option<u64>,
}

pub struct CgroupManager {
    cgroup_path: PathBuf,
}

impl CgroupManager {
    pub fn new(sandbox_id: &str) -> Result<Self, JarError> {
        let base_cgroup = Path::new("/sys/fs/cgroup");
        let cgroup_path = base_cgroup.join("jar").join(sandbox_id);

        if !base_cgroup.exists() {
            return Err(JarError::Validation(
                "Cgroups v2 unified hierarchy (/sys/fs/cgroup) is not mounted on host".to_string(),
            ));
        }

        create_dir_all(&cgroup_path)?;

        Ok(CgroupManager { cgroup_path })
    }

    pub fn apply_limits(&self, limits: &ResourceLimits) -> Result<(), JarError> {
        if let Some(mem_max) = limits.memory_max_bytes {
            self.write_cgroup_file("memory.max", &mem_max.to_string())?;
        }

        if let (Some(quota), Some(period)) = (limits.cpu_max_quota_us, limits.cpu_max_period_us) {
            self.write_cgroup_file("cpu.max", &format!("{} {}", quota, period))?;
        }

        if let Some(pids_max) = limits.pids_max {
            self.write_cgroup_file("pids.max", &pids_max.to_string())?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn attach_process(&self, pid: Pid) -> Result<(), JarError> {
        self.write_cgroup_file("cgroup.procs", &pid.as_raw().to_string())
    }

    fn write_cgroup_file(&self, filename: &str, content: &str) -> Result<(), JarError> {
        let file_path = self.cgroup_path.join(filename);
        let mut file = File::create(&file_path).map_err(|e| {
            JarError::Execution(format!("Failed to open cgroup file {:?}: {}", file_path, e))
        })?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

impl Drop for CgroupManager {
    fn drop(&mut self) {
        let _ = remove_dir(&self.cgroup_path);
    }
}
