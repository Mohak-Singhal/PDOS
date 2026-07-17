use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use crate::constants;
use crate::models::{
    Capability,
    DeviceType,
    Node,
    OperatingSystem,
};

// Nodes are considered offline if they haven't been seen for 15 seconds.
const NODE_TIMEOUT: Duration = Duration::from_secs(constants::NODE_TIMEOUT_SECS);

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
        println!("Registry initialized");
    }

    pub fn upsert_node(&mut self, node: Node) {
        if self.nodes.contains_key(&node.id) {
            println!("Updated device: {}", node.name);
        } else {
            println!("New device discovered: {}", node.name);
        }
    
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn remove_stale_nodes(&mut self) {
        let now = SystemTime::now();

        self.nodes.retain(|_, node| {
            let alive = match now.duration_since(node.last_seen) {
                Ok(elapsed) => elapsed < NODE_TIMEOUT,
                Err(_) => true,
            };

            if !alive {
                println!("Node timed out: {}", node.name);
            }

            alive
        });
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
    pub fn find_by_capability(
        &self,
        capability: Capability,
    ) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| {
                node.capabilities.contains(&capability)
            })
            .collect()
    }
    pub fn find_by_device_type(
        &self,
        device_type: DeviceType,
    ) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| node.device_type == device_type)
            .collect()
    }
    pub fn find_by_operating_system(
        &self,
        operating_system: OperatingSystem,
    ) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| node.operating_system == operating_system)
            .collect()
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