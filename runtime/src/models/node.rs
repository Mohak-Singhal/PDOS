use crate::models::NodeState;
use crate::{
    models::{Capability, DeviceType, OperatingSystem},
    protocol::{DiscoverMessage, HeartbeatMessage, Packet},
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

    pub state: NodeState,

    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub last_heartbeat: SystemTime,
}
impl Node {
    pub fn from_discovery(message: &DiscoverMessage, sender: SocketAddr) -> Self {
        let now = SystemTime::now();

        Self {
            id: message.identity.node_id.clone(),
            name: message.device.hostname.clone(),

            device_type: message.device.device_type.clone(),
            operating_system: message.device.operating_system.clone(),

            capabilities: message.capabilities.clone(),

            ip: sender.ip(),
            port: message.network.http_port,

            state: NodeState::Discovered,

            first_seen: now,
            last_seen: now,
            last_heartbeat: now,
        }
    }
    pub fn update_from_discovery(&mut self, other: &Node) {
        self.name = other.name.clone();

        self.device_type = other.device_type.clone();
        self.operating_system = other.operating_system.clone();

        self.ip = other.ip;
        self.port = other.port;

        self.capabilities = other.capabilities.clone();

        self.last_seen = other.last_seen;

        self.state = NodeState::Alive;
    }
}
