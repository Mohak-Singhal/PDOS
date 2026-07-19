use tokio::time::{Duration, interval};

use crate::constants;
use crate::discovery::Discovery;
use crate::identity::Identity;
use crate::protocol::{DiscoverMessage, Packet};
use crate::models::{
    IdentityInfo,
    NetworkInfo,
    RuntimeInfo,
    Node,
};
use crate::registry::Registry;
use crate::security;
use crate::system::SystemCollector;
use crate::transport::Transport;
use crate::liveness::Liveness;
use super::Config;
use crate::events::RuntimeEvent;
use super::RuntimeEventLoop; //use crate::runtime::RuntimeEventLoop;
use super::RuntimeContext;
pub struct Runtime {
    config: Config,
    identity: Identity,

    registry: Registry,
    transport: Transport,
    discovery: Discovery,
    liveness: Liveness,
    event_loop: RuntimeEventLoop,
}
impl Runtime {
    pub fn new(config: Config, identity: Identity) -> Self {
        Self {
            config,
            identity,
            registry: Registry::new(),
            transport: Transport::new(),
            discovery: Discovery::new(),
            liveness: Liveness::new(),
            event_loop: RuntimeEventLoop::new(),
        }
    }

    async fn send_packet(&self, packet: Packet) {
        self.transport.send(&packet).await;
    }

    fn create_discover_message(&self) -> DiscoverMessage {
        DiscoverMessage::new(
            IdentityInfo {
                node_id: self.identity.node_id.clone(),
            },
            SystemCollector::collect(
                self.config.device_type.clone(),
                self.config.operating_system.clone(),
            ),
            RuntimeInfo {
                runtime_version: env!("CARGO_PKG_VERSION").to_string(),
                protocol_version: constants::PROTOCOL_VERSION,
            },
            NetworkInfo {
                http_port: self.config.http_port,
            },
            self.config.capabilities.clone(),
            crate::utils::time::unix_timestamp(),
        )
    }

    
    pub async fn start(&mut self) {
        let mut heartbeat = interval(self.liveness.interval());

        println!("Runtime starting...");
        println!("Node ID: {}", self.identity.node_id);

        let discover = self.create_discover_message();

        println!("\nDiscoverMessage:");
        println!("{}", serde_json::to_string_pretty(&discover).unwrap());

        security::init();

        self.registry.initialize();

        self.transport.initialize().await;
        self.discovery.initialize();

        println!("Runtime ready.");
        println!("Waiting for discovery packets...");

        // Send discovery ONCE at startup.
        self.send_packet(Packet::Discover(discover)).await;

        // // Advertise immediately
        // self.send_packet(Packet::Discover(discover)).await;

        loop {
            tokio::select! {

                // Broadcast every heartbeat interval.
                _ = heartbeat.tick() => {

                    let packet = self.liveness.heartbeat_tick(
                        &self.identity,
                        &mut self.registry,
                    );
                    
                    self.send_packet(packet).await;
                }

                result = self.transport.receive() => {

                    if let Some((packet, sender)) = result {

                        self.event_loop.dispatch(
                            RuntimeEvent::NetworkPacket {
                                packet,
                                sender,
                            },
                            RuntimeContext {
                                discovery: &mut self.discovery,
                                liveness: &self.liveness,
                                registry: &mut self.registry,
                                local_node_id: &self.identity.node_id,
                            },
                        );
                        
                        self.registry.update_node_states();
                    }
                }
            }
        }
    }
    pub fn local_node_id(&self) -> &str {
        &self.identity.node_id
    }
    pub fn runtime_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    pub fn protocol_version(&self) -> u16 {
        constants::PROTOCOL_VERSION
    }
    pub fn nodes(&self) -> Vec<&Node> {
        self.registry.list_nodes()
    }
    
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.registry.get_node(id)
    }
    
    pub fn online_nodes(&self) -> Vec<&Node> {
        self.registry.online_nodes()
    }
}
