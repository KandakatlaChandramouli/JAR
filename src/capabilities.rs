use crate::error::JarError;
use caps::{clear, CapSet};

pub struct CapabilityManager;

impl CapabilityManager {
    pub fn drop_all_capabilities() -> Result<(), JarError> {
        // Clear Effective, Inheritable, Permitted, Bounding, and Ambient capability sets
        clear(None, CapSet::Effective)
            .map_err(|e| JarError::Execution(format!("Failed to clear effective caps: {}", e)))?;
        clear(None, CapSet::Permitted)
            .map_err(|e| JarError::Execution(format!("Failed to clear permitted caps: {}", e)))?;
        clear(None, CapSet::Inheritable)
            .map_err(|e| JarError::Execution(format!("Failed to clear inheritable caps: {}", e)))?;

        #[cfg(target_os = "linux")]
        {
            let _ = clear(None, CapSet::Ambient);
        }

        Ok(())
    }
}
