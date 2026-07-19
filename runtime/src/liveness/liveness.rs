use std::time::Duration;

use crate::{
    constants,
    identity::Identity,
    protocol::{HeartbeatMessage, Packet},
    registry::Registry,
};

pub struct Liveness {
    heartbeat_interval: Duration,
}

impl Liveness {
    pub fn new() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(
                constants::HEARTBEAT_INTERVAL_SECS,
            ),
        }
    }

    pub fn interval(&self) -> Duration {
        self.heartbeat_interval
    }

    fn create_heartbeat(
        &self,
        identity: &Identity,
    ) -> HeartbeatMessage {
        HeartbeatMessage::new(
            identity.node_id.clone(),
            crate::utils::time::unix_timestamp(),
        )
    }

    pub fn heartbeat_tick(
        &self,
        identity: &Identity,
        registry: &mut Registry,
    ) -> Packet {
        // Update node liveness before sending our heartbeat.
        registry.update_node_states();

        Packet::Heartbeat(self.create_heartbeat(identity))
    }
}