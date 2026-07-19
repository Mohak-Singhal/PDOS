use crate::constants;
use crate::models::{Capability, DeviceType, OperatingSystem};

#[derive(Debug, Clone)]
pub struct Config {
    pub device_type: DeviceType,
    pub operating_system: OperatingSystem,

    pub udp_port: u16,
    pub http_port: u16,

    pub capabilities: Vec<Capability>,
}

impl Default for Config {
    fn default() -> Self {
        let platform = crate::system::detect();

        Self {
            device_type: platform.device_type,
            operating_system: platform.operating_system,

            udp_port: constants::MULTICAST_PORT,
            http_port: constants::DEFAULT_HTTP_PORT,

            capabilities: vec![
                Capability::Clipboard,
                Capability::Notifications,
                Capability::Terminal,
                Capability::FileTransfer,
            ],
        }
    }
}
