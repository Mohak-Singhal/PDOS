pub mod constants;
pub mod discovery;
pub mod events;
pub mod ffi;
pub mod identity;
pub mod liveness;
pub mod models;
pub mod protocol;
pub mod registry;
pub mod runtime;
pub mod security;
pub mod system;
pub mod transport;
pub mod utils;

use identity::Identity;
use runtime::{Config, Runtime};
use std::sync::{Once, OnceLock};

pub(crate) static APP_DATA_DIR: OnceLock<String> = OnceLock::new();
static LOG_INIT: Once = Once::new();

pub fn init_logging() {
    LOG_INIT.call_once(|| {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("PDOS-RUST"),
        );

        #[cfg(not(target_os = "android"))]
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp(None)
            .init();
    });
}

pub async fn start_runtime() {
    init_logging();
    log::info!("==============================");
    log::info!(" PDOS Runtime v0.1");
    log::info!("==============================");

    let config = Config::default();
    let identity = Identity::load();

    let mut runtime = Runtime::new(config, identity);

    runtime.start().await;
}