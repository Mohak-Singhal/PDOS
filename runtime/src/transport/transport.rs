use crate::constants;
use crate::protocol::Packet;
use tokio::net::UdpSocket;

pub struct Transport {
    udp_socket: Option<UdpSocket>,
}

impl Transport {
    pub fn new() -> Self {
        Self { udp_socket: None }
    }

    pub async fn initialize(&mut self) {
        use std::net::{Ipv4Addr, UdpSocket as StdUdpSocket};

        let std_socket = StdUdpSocket::bind(format!("0.0.0.0:{}", constants::MULTICAST_PORT))
            .expect("Failed to bind UDP socket");

        std_socket
            .join_multicast_v4(
                &constants::MULTICAST_GROUP
                    .parse::<Ipv4Addr>()
                    .expect("Invalid multicast group"),
                &Ipv4Addr::UNSPECIFIED,
            )
            .expect("Failed to join multicast group");

        std_socket
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");

        let socket = UdpSocket::from_std(std_socket).expect("Failed to create Tokio socket");

        self.udp_socket = Some(socket);

        println!("UDP socket listening on {}", constants::MULTICAST_PORT);
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

        println!("Received {} bytes from {}", size, sender);

        let json = match std::str::from_utf8(&buffer[..size]) {
            Ok(value) => value,
            Err(_) => {
                println!("Invalid UTF-8 packet");
                return None;
            }
        };

        println!("{}", json);

        let packet = match serde_json::from_str::<Packet>(json) {
            Ok(packet) => packet,
            Err(err) => {
                println!("Invalid packet: {}", err);
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

        let address = format!(
            "{}:{}",
            constants::MULTICAST_GROUP,
            constants::MULTICAST_PORT
        );

        socket
            .send_to(&data, &address)
            .await
            .expect("Failed to send multicast packet");

        println!("Multicasted {} bytes", data.len());
    }
}
