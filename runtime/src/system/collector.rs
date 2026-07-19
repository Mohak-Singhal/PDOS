use hostname::get;
use sysinfo::System;

use crate::models::{DeviceInfo, DeviceType, OperatingSystem};

pub struct SystemCollector;

impl SystemCollector {
    pub fn collect(device_type: DeviceType, operating_system: OperatingSystem) -> DeviceInfo {
        let mut system = System::new_all();
        system.refresh_all();

        let hostname = get().unwrap_or_default().to_string_lossy().to_string();

        let os_version = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());

        let architecture = std::env::consts::ARCH.to_string();

        let cpu_model = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let cpu_cores = system.cpus().len() as u16;

        // sysinfo returns bytes
        let total_memory_mb = system.total_memory() / (1024 * 1024);

        DeviceInfo {
            hostname,
            device_type,
            operating_system,
            os_version,
            architecture,
            cpu_model,
            cpu_cores,
            total_memory_mb,
        }
    }
}
