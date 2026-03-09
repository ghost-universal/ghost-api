//! Ghost API Server
//!
//! HTTP server with Swagger UI for the Ghost API.

use std::net::SocketAddr;

use ghost_server::{create_app, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement server startup
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ServerConfig::from_env();
    let app = create_app(config).await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Ghost API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
