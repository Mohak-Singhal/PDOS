use serde::{Deserialize, Serialize};

use crate::models::{DeviceType, OperatingSystem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub hostname: String,

    pub device_type: DeviceType,

    pub operating_system: OperatingSystem,

    pub os_version: String,

    pub architecture: String,

    pub cpu_model: String,

    pub cpu_cores: u16,

    pub total_memory_mb: u64,
}
