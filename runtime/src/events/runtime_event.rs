use std::net::SocketAddr;

use crate::protocol::Packet;

#[derive(Debug)]
pub enum RuntimeEvent {
    NetworkPacket {
        packet: Packet,
        sender: SocketAddr,
    },

    HeartbeatTick,
}