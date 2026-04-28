mod config;
mod license;
mod signaling_http;
mod ws_router;

use std::collections::HashMap;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::RwLock;

use config::Config;
use signaling_http::RoomMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::parse();

    tracing::info!("========================================");
    tracing::info!("Hollow Signaling + WebSocket Relay");
    tracing::info!("Port: {} | TLS: {}", config.port, if config.no_tls { "off" } else { "on" });
    tracing::info!("========================================");

    let license_state = match license::LicenseState::load_from_file(
        std::path::Path::new(&config.keys_file),
    ) {
        Ok(state) => {
            tracing::info!("License keys loaded from {}", config.keys_file);
            Arc::new(state)
        }
        Err(e) => {
            tracing::warn!("No keys file ({e}), license keys disabled");
            Arc::new(license::LicenseState::disabled())
        }
    };

    let rooms: RoomMap = Arc::new(RwLock::new(HashMap::new()));
    let ws_state = Arc::new(ws_router::WsState::new(license_state.clone()));
    license_state.clone().spawn_reload_task(ws_state.clone());

    tokio::select! {
        result = async {
            if config.no_tls {
                signaling_http::run_signaling_http(
                    rooms, ws_state, license_state, config.port,
                ).await
            } else {
                signaling_http::run_signaling_tls(
                    rooms, ws_state, license_state, config.port,
                    config.tls_cert, config.tls_key,
                ).await
            }
        } => {
            tracing::error!("Server exited: {result:?}");
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
    }

    Ok(())
}
