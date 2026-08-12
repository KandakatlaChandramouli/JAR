use crate::error::JarError;
use flate2::read::GzDecoder;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use tar::Archive;

pub struct ImageUnpacker;

impl ImageUnpacker {
    pub fn unpack_tarball(image_path: &str, sandbox_id: &str) -> Result<PathBuf, JarError> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err(JarError::Validation(format!(
                "Image tarball not found: {}",
                image_path
            )));
        }

        let target_dir = Path::new("/tmp").join("jar_images").join(sandbox_id);
        create_dir_all(&target_dir)?;

        let file = File::open(path)
            .map_err(|e| JarError::Execution(format!("Failed to open image tarball: {}", e)))?;

        println!("[jar] unpacking OCI container image layer into rootfs cache");

        if image_path.ends_with(".gz") || image_path.ends_with(".tgz") {
            let gz = GzDecoder::new(file);
            let mut archive = Archive::new(gz);
            archive.unpack(&target_dir).map_err(|e| {
                JarError::Execution(format!("Failed to unpack compressed tarball: {}", e))
            })?;
        } else {
            let mut archive = Archive::new(file);
            archive.unpack(&target_dir).map_err(|e| {
                JarError::Execution(format!("Failed to unpack image tarball: {}", e))
            })?;
        }

        Ok(target_dir)
    }
}
