use std::net::SocketAddr;

#[derive(Debug)]
pub enum RuntimeEvent {
    NetworkPacket {
        bytes: Vec<u8>,
        sender: SocketAddr,
    },
}