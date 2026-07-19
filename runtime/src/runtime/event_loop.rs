

use crate::{
    discovery::Discovery,
    events::RuntimeEvent,
    liveness::Liveness,
    registry::Registry,
};
use super::RuntimeContext; //use crate::runtime::RuntimeContext;
pub struct RuntimeEventLoop;

impl RuntimeEventLoop {
    pub fn new() -> Self {
        Self
    }

    pub fn dispatch(
        &self,
        event: RuntimeEvent,
        mut context: RuntimeContext,
    ) {
        match event {
    
            RuntimeEvent::NetworkPacket {
                packet,
                sender,
            } => {
    
                context.discovery.handle_packet(
                    packet,
                    sender,
                    context.local_node_id,
                    context.registry,
                );
    
            }
    
            RuntimeEvent::HeartbeatTick => {
                // next
            }
    
        }
    }
}