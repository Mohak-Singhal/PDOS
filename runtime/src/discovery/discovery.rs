pub struct Discovery;

impl Discovery {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(&self) {
        println!("Discovery initialized");
    }

    pub fn handle_packet(
        &self,
        bytes: &[u8],
        sender: std::net::SocketAddr,
    ) {
        println!(
            "Discovery received {} bytes from {}",
            bytes.len(),
            sender
        );
    }
}