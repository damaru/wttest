use anyhow::Result;
use clap::Parser;
use quinn::Endpoint;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

mod config;
mod demo;

use config::configure_client;
use demo::{demo_bidirectional, demo_datagram, demo_unidirectional};

#[derive(Parser, Debug)]
#[command(name = "webtransport-client")]
#[command(about = "A WebTransport client implementation")]
struct Args {
    /// Server address
    #[arg(short, long, default_value = "127.0.0.1:4433")]
    server: SocketAddr,

    /// Server name for SNI
    #[arg(short, long, default_value = "localhost")]
    name: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Skip certificate verification (for self-signed certs)
    #[arg(long, default_value = "true")]
    insecure: bool,

    /// Demo mode to run
    #[arg(short, long, default_value = "all")]
    demo: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Connecting to WebTransport server at {}", args.server);

    // Configure client
    let client_config = configure_client(args.insecure)?;
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    // Connect to server
    let connection = endpoint
        .connect(args.server, &args.name)?
        .await?;

    info!("Connected to server");

    let connection = Arc::new(connection);

    // Run demos based on the specified mode
    match args.demo.as_str() {
        "bidirectional" | "bidi" => {
            demo_bidirectional(connection.clone()).await?;
        }
        "unidirectional" | "uni" => {
            demo_unidirectional(connection.clone()).await?;
        }
        "datagram" | "dgram" => {
            demo_datagram(connection.clone()).await?;
        }
        "all" => {
            info!("Running all demos...");
            
            // Run bidirectional demo
            if let Err(err) = demo_bidirectional(connection.clone()).await {
                error!("Bidirectional demo failed: {}", err);
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Run unidirectional demo
            if let Err(err) = demo_unidirectional(connection.clone()).await {
                error!("Unidirectional demo failed: {}", err);
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Run datagram demo
            if let Err(err) = demo_datagram(connection.clone()).await {
                error!("Datagram demo failed: {}", err);
            }
        }
        _ => {
            error!("Unknown demo mode: {}", args.demo);
            error!("Available modes: bidirectional, unidirectional, datagram, all");
            return Ok(());
        }
    }

    // Keep connection alive for a bit
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Close connection gracefully
    connection.close(0u32.into(), b"demo completed");
    
    // Wait for connection to close
    connection.closed().await;
    info!("Connection closed");

    Ok(())
}
