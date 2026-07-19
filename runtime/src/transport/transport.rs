use crate::constants;
use crate::protocol::Packet;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{Ipv4Addr, UdpSocket as StdUdpSocket};
use tokio::net::UdpSocket;

pub struct Transport {
    udp_socket: Option<UdpSocket>,
}

impl Transport {
    pub fn new() -> Self {
        Self { udp_socket: None }
    }

    fn find_local_ip() -> Option<Ipv4Addr> {
        // Bind a temporary UDP socket and "connect" to a remote address.
        // The kernel selects the best source IP for the route without sending packets.
        let temp = StdUdpSocket::bind("0.0.0.0:0").ok()?;
        temp.connect("1.1.1.1:53").ok()?;
        if let Ok(local) = temp.local_addr() {
            if let std::net::IpAddr::V4(ip) = local.ip() {
                if !ip.is_loopback() {
                    return Some(ip);
                }
            }
        }
        None
    }

    pub async fn initialize(&mut self) {
        let multicast_addr: Ipv4Addr = constants::MULTICAST_GROUP
            .parse()
            .expect("Invalid multicast group");

        let socket = Socket::new(
            Domain::IPV4,
            Type::DGRAM,
            Some(Protocol::UDP),
        )
        .expect("Failed to create socket");

        socket
            .bind(&SockAddr::from(std::net::SocketAddrV4::new(
                Ipv4Addr::UNSPECIFIED,
                constants::MULTICAST_PORT,
            )))
            .expect("Failed to bind UDP socket");

        socket
            .join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED)
            .expect("Failed to join multicast group");

        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.set_multicast_ttl_v4(2).expect("Failed to set multicast TTL");
        socket.set_multicast_loop_v4(true).expect("Failed to set multicast loopback");

        // Explicitly set outgoing multicast interface to a non-loopback IPv4 address
        if let Some(local_ip) = Self::find_local_ip() {
            socket
                .set_multicast_if_v4(&local_ip)
                .unwrap_or_else(|_| {
                    log::info!("Could not set multicast IF to {}, using default", local_ip);
                });
            log::info!("Set multicast interface to {}", local_ip);
        }

        socket
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");

        let std_socket: StdUdpSocket = socket.into();
        let tokio_socket = UdpSocket::from_std(std_socket).expect("Failed to create Tokio socket");

        self.udp_socket = Some(tokio_socket);

        log::info!("UDP socket listening on {}", constants::MULTICAST_PORT);
    }

    pub async fn receive(&self) -> Option<(Packet, std::net::SocketAddr)> {
        let socket = self
            .udp_socket
            .as_ref()
            .expect("UDP socket not initialized");

        let mut buffer = [0u8; 4096];

        let (size, sender) = socket
            .recv_from(&mut buffer)
            .await
            .expect("Failed to receive UDP packet");

        log::info!("Received {} bytes from {}", size, sender);

        let json = match std::str::from_utf8(&buffer[..size]) {
            Ok(value) => value,
            Err(_) => {
                log::info!("Invalid UTF-8 packet");
                return None;
            }
        };

        log::info!("{}", json);

        let packet = match serde_json::from_str::<Packet>(json) {
            Ok(packet) => packet,
            Err(err) => {
                log::info!("Invalid packet: {}", err);
                return None;
            }
        };

        Some((packet, sender))
    }

    pub async fn send(&self, packet: &Packet) {
        let socket = self
            .udp_socket
            .as_ref()
            .expect("UDP socket not initialized");

        let data = serde_json::to_vec(packet).expect("Failed to serialize packet");

        let multicast_addr = format!(
            "{}:{}",
            constants::MULTICAST_GROUP,
            constants::MULTICAST_PORT
        );

        socket
            .send_to(&data, &multicast_addr)
            .await
            .expect("Failed to send multicast packet");

        // Also broadcast to the subnet as a fallback for networks that block multicast
        let broadcast_addr = format!("255.255.255.255:{}", constants::MULTICAST_PORT);
        let _ = socket.send_to(&data, &broadcast_addr).await;

        log::info!("Multicasted {} bytes", data.len());
    }
}
