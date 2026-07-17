#[derive(Debug, Clone)]
pub struct Config {
    pub node_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub operating_system: OperatingSystem,
    pub udp_port: u16,
    pub http_port: u16,
    pub capabilities: Vec<Capability>,
}
use crate::constants;
use crate::models::{
    Capability,
    DeviceType,
    OperatingSystem,
};
impl Default for Config {
    fn default() -> Self {
        Self {
            node_id: "mac-dev-001".to_string(),
            device_name: "Mohak Mac".to_string(),
        
            device_type: DeviceType::Laptop,
            operating_system: OperatingSystem::MacOS,
        
            udp_port: 53317,
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