use crate::error::JarError;
use caps::{clear, CapSet};

pub struct CapabilityManager;

impl CapabilityManager {
    pub fn drop_all_capabilities() -> Result<(), JarError> {
        for capset in [
            CapSet::Effective,
            CapSet::Permitted,
            CapSet::Inheritable,
            CapSet::Ambient,
        ] {
            let _ = clear(None, capset);
        }
        Ok(())
    }
}
