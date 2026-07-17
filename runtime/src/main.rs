mod runtime;
mod discovery;
mod registry;
mod protocol;
mod transport;
mod security;
mod models;
mod events;
mod constants;

use runtime::{Config, Runtime};

#[tokio::main]
async fn main() {
    println!("==============================");
    println!(" PDOS Runtime v0.1");
    println!("==============================");

    let config = Config::default();

    let mut runtime = Runtime::new(config);

    runtime.start().await;
}