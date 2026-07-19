
use crate::constants::{OFFLINE_TIMEOUT_SECS, SUSPECT_TIMEOUT_SECS};
use crate::models::{Capability, DeviceType, Node, NodeState, OperatingSystem};
use std::collections::HashMap;
use std::time::SystemTime;
pub struct Registry {
    nodes: HashMap<String, Node>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn initialize(&self) {
        log::info!("Registry initialized");
    }

    pub fn upsert_node(&mut self, node: Node) {
        if let Some(existing) = self.nodes.get_mut(&node.id) {
            // Compare first

            if existing.ip != node.ip {
                println!("IP changed: {} -> {}", existing.ip, node.ip);
            }

            if existing.name != node.name {
                println!("Hostname changed: {} -> {}", existing.name, node.name);
            }

            if existing.capabilities != node.capabilities {
                println!("Capabilities updated.");
            }

            // Then update fields
            let recovered = existing.state == NodeState::Offline;

            existing.update_from_discovery(&node);
            if recovered {
                log::info!("Device came back online: {}", existing.name);
            }
        } else {
            log::info!("New device discovered: {}", node.name);

            self.nodes.insert(node.id.clone(), node);
        }
    }

    pub fn update_node_states(&mut self) {
        let now = SystemTime::now();

        for node in self.nodes.values_mut() {
            let elapsed = now
                .duration_since(node.last_heartbeat)
                .unwrap_or_default()
                .as_secs();

            if elapsed >= OFFLINE_TIMEOUT_SECS {
                node.state = NodeState::Offline;
            } else if elapsed >= SUSPECT_TIMEOUT_SECS {
                node.state = NodeState::Suspect;
            } else {
                node.state = NodeState::Alive;
            }
        }
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }
    pub fn count(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn contains(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn list_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }
    pub fn online_nodes(&self) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| node.state == NodeState::Alive)
            .collect()
    }
    pub fn find_by_capability(&self, capability: Capability) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| node.capabilities.contains(&capability))
            .collect()
    }
    pub fn find_by_device_type(&self, device_type: DeviceType) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| node.device_type == device_type)
            .collect()
    }
    pub fn find_by_operating_system(&self, operating_system: OperatingSystem) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| node.operating_system == operating_system)
            .collect()
    }
    pub fn heartbeat(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            let now = SystemTime::now();

            node.last_seen = now;
            node.last_heartbeat = now;
            node.state = NodeState::Alive;
        }
    }

    pub fn print_nodes(&self) {
        println!();
        println!("==============================");
        println!("Discovered Devices");
        println!("==============================");

        if self.nodes.is_empty() {
            println!("No devices discovered.");
            return;
        }

        for node in self.nodes.values() {
            println!("Name      : {}", node.name);
            println!("Node ID   : {}", node.id);
            println!("IP        : {}", node.ip);
            println!("Port      : {}", node.port);

            println!("Type      : {:?}", node.device_type);
            println!("OS        : {:?}", node.operating_system);

            println!("Capabilities:");
            for capability in &node.capabilities {
                println!("  • {:?}", capability);
            }

            println!("Last Seen : {:?}", node.last_seen);
            println!();
        }
    }
}
