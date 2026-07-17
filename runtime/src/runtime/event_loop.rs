use crate::events::RuntimeEvent;

pub struct RuntimeEventLoop;

impl RuntimeEventLoop {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(&mut self, event: RuntimeEvent) {
        match event {
            RuntimeEvent::NetworkPacket { bytes, sender } => {
                println!(
                    "Runtime Event: {} bytes from {}",
                    bytes.len(),
                    sender
                );
            }
        }
    }
}