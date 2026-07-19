mod constants;
mod discovery;
mod events;
mod identity;
mod models;
mod protocol;
mod registry;
mod runtime;
mod security;
mod system;
mod transport;
mod utils;
mod liveness;

use identity::Identity;
use runtime::{Config, Runtime};

#[tokio::main]
async fn main() {
    println!("==============================");
    println!(" PDOS Runtime v0.1");
    println!("==============================");

    let config = Config::default();

    let identity = Identity::load();

    let mut runtime = Runtime::new(config, identity);

    runtime.start().await;
}
