use std::net::SocketAddr;
use crate::models::Node;
use crate::{
    protocol::{DiscoverMessage, HeartbeatMessage, Packet},
    registry::Registry,
};

pub struct Discovery;

impl Discovery {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(&self) {
        println!("Discovery initialized");
    }

    pub fn handle_packet(
        &self,
        packet: Packet,
        sender: SocketAddr,
        local_node_id: &str,
        registry: &mut Registry,
    ) {
        match packet {
            Packet::Discover(message) => {
                self.handle_discover(message, sender, local_node_id, registry);
            }

            Packet::Heartbeat(message) => {
                self.handle_heartbeat(message, registry);
            }
        }
    }

    fn handle_discover(
        &self,
        message: DiscoverMessage,
        sender: SocketAddr,
        local_node_id: &str,
        registry: &mut Registry,
    ) {
        // 1. Validate packet
        if !message.is_valid() {
            println!("Ignoring invalid discovery packet.");
            return;
        }
    
        // 2. Ignore our own packets
        if message.identity.node_id == local_node_id {
            return;
        }
    
        // 3. Check protocol compatibility
        if message.runtime.protocol_version != crate::constants::PROTOCOL_VERSION {
            println!(
                "Ignoring incompatible protocol version: {}",
                message.runtime.protocol_version
            );
            return;
        }
    
        // 4. Convert packet into runtime node
        let node = Node::from_discovery(&message, sender);
    
        // 5. Update registry
        registry.upsert_node(node);
    }

    fn handle_heartbeat(
        &self,
        message: HeartbeatMessage,
        registry: &mut Registry,
    ) {
        // Validate heartbeat
        if !message.is_valid() {
            println!("Ignoring invalid heartbeat.");
            return;
        }
    
        // Refresh node heartbeat
        registry.heartbeat(&message.node_id);
    }
}