//! Ghost API Server
//!
//! HTTP server with Swagger UI for the Ghost API.
//! Provides REST endpoints for interacting with social media platforms.

use std::net::SocketAddr;
use std::sync::Arc;

use ghost_server::{create_app, ServerConfig};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();

    // Load configuration from environment
    let config = ServerConfig::from_env();

    // Validate configuration
    if let Err(e) = config.validate() {
        tracing::error!("Configuration error: {}", e);
        std::process::exit(1);
    }

    tracing::info!(
        addr = %config.addr,
        cors = config.cors_enabled,
        swagger = config.swagger_enabled,
        timeout_secs = config.request_timeout_secs,
        "Starting Ghost API server"
    );

    // Create the Axum application
    let app = create_app(config.clone()).await?;

    // Bind to the configured address
    let listener = tokio::net::TcpListener::bind(config.addr).await?;

    tracing::info!(
        addr = %config.addr,
        "Ghost API server listening"
    );

    tracing::info!(
        swagger_url = format!("http://{}/swagger-ui/", config.addr),
        "Swagger UI available"
    );

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Ghost API server shutdown complete");

    Ok(())
}

/// Creates a graceful shutdown signal handler
///
/// Handles both SIGINT (Ctrl+C) and SIGTERM signals.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, initiating graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, initiating graceful shutdown");
        }
    }
}
