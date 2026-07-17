use serde::{Deserialize, Serialize};

use crate::models::{
    Capability,
    DeviceType,
    OperatingSystem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverMessage {
    pub protocol_version: u16,

    pub node_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub operating_system: OperatingSystem,

    pub capabilities: Vec<Capability>,

    pub http_port: u16,
}

impl DiscoverMessage {
    pub fn new(
        protocol_version: u16,
        node_id: String,
        device_name: String,
        device_type: DeviceType,
        operating_system: OperatingSystem,
        capabilities: Vec<Capability>,
        http_port: u16,
    ) -> Self {
        Self {
            protocol_version,
            node_id,
            device_name,
            device_type,
            operating_system,
            capabilities,
            http_port,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.protocol_version != 1 {
            return false;
        }
    
        if self.node_id.trim().is_empty() {
            return false;
        }
    
        if self.device_name.trim().is_empty() {
            return false;
        }
    
        if self.http_port == 0 {
            return false;
        }
    
        true
    }
}