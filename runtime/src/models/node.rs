use crate::{
    models::{
        Capability,
        DeviceType,
        OperatingSystem,
    },
    protocol::DiscoverMessage,
};

use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,

    pub device_type: DeviceType,
    pub operating_system: OperatingSystem,

    pub capabilities: Vec<Capability>,

    pub ip: IpAddr,
    pub port: u16,
    pub last_seen: SystemTime,
}
impl Node {
    pub fn from_discovery(
        message: DiscoverMessage,
        sender: SocketAddr,
    ) -> Self {
        Self {
            id: message.node_id,
            name: message.device_name,
        
            device_type: message.device_type,
            operating_system: message.operating_system,
            capabilities: message.capabilities,
        
            ip: sender.ip(),
            port: message.http_port,
            last_seen: SystemTime::now(),
        }
    }
}