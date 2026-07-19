use serde::{Deserialize, Serialize};

use crate::models::{Capability, DeviceInfo, IdentityInfo, NetworkInfo, RuntimeInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverMessage {
    pub identity: IdentityInfo,
    pub device: DeviceInfo,
    pub runtime: RuntimeInfo,
    pub network: NetworkInfo,
    pub capabilities: Vec<Capability>,
    pub timestamp: u64,
}

impl DiscoverMessage {
    pub fn new(
        identity: IdentityInfo,
        device: DeviceInfo,
        runtime: RuntimeInfo,
        network: NetworkInfo,
        capabilities: Vec<Capability>,
        timestamp: u64,
    ) -> Self {
        Self {
            identity,
            device,
            runtime,
            network,
            capabilities,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.identity.node_id.trim().is_empty() {
            return false;
        }

        if self.device.hostname.trim().is_empty() {
            return false;
        }

        if self.network.http_port == 0 {
            return false;
        }
        if self.timestamp == 0 {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub node_id: String,
    pub timestamp: u64,
}

impl HeartbeatMessage {
    pub fn new(node_id: String, timestamp: u64) -> Self {
        Self { node_id, timestamp }
    }

    pub fn is_valid(&self) -> bool {
        !self.node_id.trim().is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Discover(DiscoverMessage),
    Heartbeat(HeartbeatMessage),
}
