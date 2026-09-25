use anyhow::Result;
use clap::Parser;
use std::time::Duration;
use tracing::{error, info};
use wtransport::Endpoint;

mod config;
mod demo;

use config::configure_client;
use demo::{demo_bidirectional, demo_datagram, demo_unidirectional};

#[derive(Parser, Debug)]
#[command(name = "webtransport-client")]
#[command(about = "A WebTransport-over-HTTP/3 client implementation")]
struct Args {
    /// WebTransport URL
    #[arg(short, long, default_value = "https://127.0.0.1:4433/")]
    url: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Skip certificate verification for self-signed development certificates
    #[arg(long, default_value = "true")]
    insecure: bool,

    /// Demo mode to run
    #[arg(short, long, default_value = "all")]
    demo: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Connecting to WebTransport server at {}", args.url);

    let client_config = configure_client(args.insecure);
    let endpoint = Endpoint::client(client_config)?;
    let connection = endpoint.connect(&args.url).await?;

    info!("Connected to server");

    match args.demo.as_str() {
        "bidirectional" | "bidi" => {
            demo_bidirectional(&connection).await?;
        }
        "unidirectional" | "uni" => {
            demo_unidirectional(&connection).await?;
        }
        "datagram" | "dgram" => {
            demo_datagram(&connection).await?;
        }
        "all" => {
            info!("Running all demos...");

            if let Err(err) = demo_bidirectional(&connection).await {
                error!("Bidirectional demo failed: {}", err);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;

            if let Err(err) = demo_unidirectional(&connection).await {
                error!("Unidirectional demo failed: {}", err);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;

            if let Err(err) = demo_datagram(&connection).await {
                error!("Datagram demo failed: {}", err);
            }
        }
        _ => {
            error!("Unknown demo mode: {}", args.demo);
            error!("Available modes: bidirectional, unidirectional, datagram, all");
            return Ok(());
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    connection.close(0u32.into(), b"demo completed");
    connection.closed().await;
    info!("Connection closed");

    Ok(())
}
