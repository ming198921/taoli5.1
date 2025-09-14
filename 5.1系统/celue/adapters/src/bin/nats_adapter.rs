//! Standalone NATS adapter binary
//! 
//! This binary runs the NATS adapter as a separate service.

use std::env;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;
use tokio::signal;

use adapters::Adapter;
use adapters::nats::{NatsAdapter, NatsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting NATS adapter");

    let config = NatsConfig {
        servers: vec!["nats://127.0.0.1:4222".to_string()],
        name: "arbitrage_adapter".to_string(),
        token: None,
        username: None,
        password: None,
        connect_timeout: Duration::from_secs(10),
        request_timeout: Duration::from_secs(5),
        enable_jetstream: true,
        jetstream_domain: None,
        streams: vec![],
        consumers: vec![],
    };

    let mut adapter = NatsAdapter::new();
    
    adapter.initialize(config).await?;
    info!("NATS adapter initialized successfully");

    adapter.start().await?;
    info!("NATS adapter started successfully");

    // Create health check task
    let health_check_task = async {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = adapter.health_check().await {
                warn!("NATS adapter health check failed: {}", e);
            } else {
                info!("NATS adapter health check passed");
            }
        }
    };

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
        },
        _ = health_check_task => {
            warn!("Health check task ended unexpectedly");
        }
    }

    info!("Shutting down NATS adapter");
    adapter.stop().await?;
    Ok(())
}
