use crate::error::JarError;
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct OverlayManager {
    pub lower_dir: PathBuf,
    pub upper_dir: PathBuf,
    pub work_dir: PathBuf,
    pub merged_dir: PathBuf,
    pub base_tmp_path: PathBuf,
}

impl OverlayManager {
    pub fn new(sandbox_id: &str, lower_path: &str) -> Result<Self, JarError> {
        let lower_dir = Path::new(lower_path).canonicalize().map_err(|e| {
            JarError::Validation(format!("Invalid lowerdir path {}: {}", lower_path, e))
        })?;

        let base_tmp_path = Path::new("/tmp").join("jar_overlay").join(sandbox_id);
        let upper_dir = base_tmp_path.join("upper");
        let work_dir = base_tmp_path.join("work");
        let merged_dir = base_tmp_path.join("merged");

        create_dir_all(&upper_dir)?;
        create_dir_all(&work_dir)?;
        create_dir_all(&merged_dir)?;

        Ok(OverlayManager {
            lower_dir,
            upper_dir,
            work_dir,
            merged_dir,
            base_tmp_path,
        })
    }

    pub fn mount_overlay(&self) -> Result<bool, JarError> {
        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            self.lower_dir.display(),
            self.upper_dir.display(),
            self.work_dir.display()
        );

        match mount(
            Some("overlay"),
            &self.merged_dir,
            Some("overlay"),
            MsFlags::empty(),
            Some(options.as_str()),
        ) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!(
                    "[jar warning] OverlayFS mount skipped ({}); proceeding with base filesystem",
                    e
                );
                Ok(false)
            }
        }
    }

    pub fn cleanup(&self) {
        let _ = umount2(&self.merged_dir, MntFlags::MNT_DETACH);
        let _ = remove_dir_all(&self.base_tmp_path);
    }
}
