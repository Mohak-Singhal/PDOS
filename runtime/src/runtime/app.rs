use tokio::time::{interval, Duration};
use crate::security;
use crate::discovery::Discovery;
use crate::protocol::DiscoverMessage;
use crate::registry::Registry;
use crate::transport::Transport;
// use crate::models::Node;
use crate::models::{Capability, Node};
use super::Config;
use crate::constants;

pub struct Runtime {
    config: Config,
    registry: Registry,
    transport: Transport,
    discovery: Discovery,
}

impl Runtime {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            registry: Registry::new(),
            transport: Transport::new(),
            discovery: Discovery::new(),
        }
    }

    fn create_discover_message(&self) -> DiscoverMessage {
        DiscoverMessage {
            protocol_version: constants::PROTOCOL_VERSION,
            node_id: self.config.node_id.clone(),
            device_name: self.config.device_name.clone(),
            device_type: self.config.device_type.clone(),
            operating_system: self.config.operating_system.clone(),
            capabilities: self.config.capabilities.clone(),
            http_port: self.config.http_port,
        }
    }

    pub async fn start(&mut self) {
        let mut heartbeat = interval(Duration::from_secs(constants::HEARTBEAT_INTERVAL_SECS));
        println!("Runtime starting...");
        println!("Device: {}", self.config.device_name);

        let packet = self.create_discover_message();

        let json = serde_json::to_string_pretty(&packet).unwrap();

        println!("\nDiscoverMessage:");
        

        let decoded: DiscoverMessage =
            serde_json::from_str(&json).unwrap();

        // println!("\nDecoded DiscoverMessage:");
        // println!("{:#?}", decoded);

        security::init();

        self.registry.initialize();

        self.transport.initialize().await;
        self.discovery.initialize();

        println!("Runtime ready.");
        println!("Waiting for discovery packets...");

        // Advertise ourselves once
        self.transport
            .multicast(json.as_bytes())
            .await;

        // Listen forever
        loop {
            tokio::select! {
        
                _ = heartbeat.tick() => {
                    let packet = self.create_discover_message();
        
                    let json = serde_json::to_string(&packet).unwrap();
        
                    self.transport
                        .multicast(json.as_bytes())
                        .await;
                }
        
                result = self.transport.receive() => {
                    if let Some((message, sender)) = result {
                
                        // Step 17: Validate discovery packet
                        if !message.is_valid() {
                            println!("Ignoring invalid discovery packet.");
                            continue;
                        }
                
                        // Step 18 (already implemented): Ignore our own packets
                        if message.node_id == self.config.node_id {
                            continue;
                        }
                
                        let node = Node::from_discovery(message, sender);
                
                        self.registry.upsert_node(node);
                        let terminals = self
                            .registry
                            .find_by_capability(Capability::Terminal);

                        println!(
                            "Terminal devices: {}",
                            terminals.len()
                        );
                        
                
                        self.registry.remove_stale_nodes();
                
                        self.registry.print_nodes();
                    }
                }
            }
        }
        

       
    }
}