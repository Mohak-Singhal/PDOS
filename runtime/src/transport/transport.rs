use crate::protocol::DiscoverMessage;
use tokio::net::UdpSocket;

pub struct Transport {
    udp_socket: Option<UdpSocket>,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            udp_socket: None,
        }
    }

    pub async fn initialize(&mut self) {
        use std::net::{Ipv4Addr, UdpSocket as StdUdpSocket};

        let std_socket = StdUdpSocket::bind("0.0.0.0:53317")
            .expect("Failed to bind UDP socket");

        std_socket
            .join_multicast_v4(
                &Ipv4Addr::new(224, 0, 0, 167),
                &Ipv4Addr::UNSPECIFIED,
            )
            .expect("Failed to join multicast group");

        std_socket
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");

        let socket = UdpSocket::from_std(std_socket)
            .expect("Failed to create Tokio socket");

        self.udp_socket = Some(socket);

        println!("UDP socket listening on 53317");
    }

    pub async fn receive(
        &self,
    ) -> Option<(DiscoverMessage, std::net::SocketAddr)> {
        let socket = self
            .udp_socket
            .as_ref()
            .expect("UDP socket not initialized");

        let mut buffer = [0u8; 2048];

        let (size, sender) = socket
            .recv_from(&mut buffer)
            .await
            .expect("Failed to receive UDP packet");

        let json = match std::str::from_utf8(&buffer[..size]) {
            Ok(value) => value,
            Err(_) => {
                println!("Invalid UTF-8 packet");
                return None;
            }
        };

        let message = match serde_json::from_str::<DiscoverMessage>(json) {
            Ok(message) => message,
            Err(err) => {
                println!("Invalid discovery packet: {}", err);
                return None;
            }
        };

        if !message.is_valid() {
            println!("Invalid discovery packet");
            return None;
        }

        Some((message, sender))
    }

    pub async fn multicast(&self, data: &[u8]) {
        let socket = self
            .udp_socket
            .as_ref()
            .expect("UDP socket not initialized");

        socket
            .send_to(data, "224.0.0.167:53317")
            .await
            .expect("Failed to send multicast packet");

        println!("Multicasted {} bytes", data.len());
    }
}