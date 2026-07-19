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

        let socket = match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to create UDP socket: {}", e);
                return;
            }
        };

        let _ = socket.set_reuse_address(true);

        let addr = SockAddr::from(std::net::SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            constants::MULTICAST_PORT,
        ));

        if let Err(e) = socket.bind(&addr) {
            log::error!("Failed to bind UDP socket: {}", e);
            return;
        }

        if let Err(e) = socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED) {
            log::warn!("Failed to join multicast group: {} (proceeding anyway)", e);
        }

        let _ = socket.set_broadcast(true);
        let _ = socket.set_multicast_ttl_v4(2);
        let _ = socket.set_multicast_loop_v4(true);

        if let Some(local_ip) = Self::find_local_ip() {
            if let Err(_) = socket.set_multicast_if_v4(&local_ip) {
                log::info!("Could not set multicast IF to {}, using default", local_ip);
            }
            log::info!("Set multicast interface to {}", local_ip);
        }

        let _ = socket.set_nonblocking(true);

        let std_socket: StdUdpSocket = socket.into();

        let tokio_socket = match UdpSocket::from_std(std_socket) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to create Tokio socket: {}", e);
                return;
            }
        };

        self.udp_socket = Some(tokio_socket);

        log::info!("UDP socket listening on {}", constants::MULTICAST_PORT);
    }

    fn subnet_broadcast(local_ip: Ipv4Addr) -> String {
        let ip_bits = u32::from(local_ip);
        let broadcast = ip_bits | 0x000000FF;
        let broadcast_ip = Ipv4Addr::from(broadcast);
        format!("{}:{}", broadcast_ip, constants::MULTICAST_PORT)
    }

    pub async fn send_to(&self, packet: &Packet, target: &str) {
        let socket = match self.udp_socket.as_ref() {
            Some(s) => s,
            None => return,
        };

        let data = match serde_json::to_vec(packet) {
            Ok(d) => d,
            Err(_) => return,
        };

        let _ = socket.send_to(&data, target).await;

        log::info!("Unicasted {} bytes to {}", data.len(), target);
    }

    pub async fn receive(&self) -> Option<(Packet, std::net::SocketAddr)> {
        let socket = self.udp_socket.as_ref()?;

        let mut buffer = [0u8; 4096];

        let (size, sender) = socket.recv_from(&mut buffer).await.ok()?;

        log::info!("Received {} bytes from {}", size, sender);

        let json = std::str::from_utf8(&buffer[..size]).ok()?;

        log::info!("{}", json);

        let packet = serde_json::from_str::<Packet>(json).ok()?;

        Some((packet, sender))
    }

    pub async fn send(&self, packet: &Packet) {
        let socket = match self.udp_socket.as_ref() {
            Some(s) => s,
            None => return,
        };

        let data = match serde_json::to_vec(packet) {
            Ok(d) => d,
            Err(_) => return,
        };

        let multicast_addr = format!(
            "{}:{}",
            constants::MULTICAST_GROUP,
            constants::MULTICAST_PORT
        );

        let _ = socket.send_to(&data, &multicast_addr).await;

        // Global broadcast fallback (some networks block multicast)
        let global_broadcast = format!("255.255.255.255:{}", constants::MULTICAST_PORT);
        let _ = socket.send_to(&data, &global_broadcast).await;

        // Subnet-directed broadcast fallback (Samsung hotspots often block the above)
        if let Some(local_ip) = Self::find_local_ip() {
            let subnet_bcast = Self::subnet_broadcast(local_ip);
            let _ = socket.send_to(&data, &subnet_bcast).await;
        }

        log::info!("Multicasted {} bytes", data.len());
    }
}
