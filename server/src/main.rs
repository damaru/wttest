use anyhow::Result;
use clap::Parser;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

mod cert;
mod handlers;

use cert::generate_self_signed_cert;
use handlers::{handle_bidirectional_stream, handle_datagram, handle_unidirectional_stream};

#[derive(Parser, Debug)]
#[command(name = "webtransport-server")]
#[command(about = "A WebTransport server implementation")]
struct Args {
    /// Listen address
    #[arg(short, long, default_value = "127.0.0.1:4433")]
    listen: SocketAddr,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Install default crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting WebTransport server on {}", args.listen);

    // Generate self-signed certificate
    let (cert, key) = generate_self_signed_cert()?;
    let cert_chain = vec![cert];
    let key_der = PrivateKeyDer::try_from(key)?;

    // Configure server
    let server_config = configure_server(cert_chain, key_der)?;
    let endpoint = Endpoint::server(server_config, args.listen)?;

    info!("Server listening on {}", args.listen);
    info!("Use the following certificate fingerprint to connect:");
    // In a real implementation, you'd print the actual fingerprint

    // Accept connections
    while let Some(conn) = endpoint.accept().await {
        let conn_future = conn;
        tokio::spawn(async move {
            match conn_future.await {
                Ok(connection) => handle_connection(connection).await,
                Err(err) => error!("Connection failed: {}", err),
            }
        });
    }

    Ok(())
}

fn configure_server(
    cert_chain: Vec<CertificateDer<'static>>,
    key_der: PrivateKeyDer<'static>,
) -> Result<ServerConfig> {
    let server_config = ServerConfig::with_single_cert(cert_chain, key_der)?;
    
    Ok(server_config)
}

async fn handle_connection(connection: quinn::Connection) {
    info!(
        "New connection from {}",
        connection.remote_address()
    );

    // Handle the connection
    let connection = Arc::new(connection);
    
    // Clone connection for different handlers
    let conn_bidi = connection.clone();
    let conn_uni = connection.clone();
    let conn_dgram = connection.clone();

    // Handle bidirectional streams
    let bidi_task = tokio::spawn(async move {
        loop {
            match conn_bidi.accept_bi().await {
                Ok((send, recv)) => {
                    tokio::spawn(handle_bidirectional_stream(send, recv));
                }
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("Connection closed by application");
                    break;
                }
                Err(err) => {
                    error!("Failed to accept bidirectional stream: {}", err);
                    break;
                }
            }
        }
    });

    // Handle unidirectional streams
    let uni_task = tokio::spawn(async move {
        loop {
            match conn_uni.accept_uni().await {
                Ok(recv) => {
                    tokio::spawn(handle_unidirectional_stream(recv));
                }
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("Connection closed by application");
                    break;
                }
                Err(err) => {
                    error!("Failed to accept unidirectional stream: {}", err);
                    break;
                }
            }
        }
    });

    // Handle datagrams
    let dgram_task = tokio::spawn(async move {
        loop {
            match conn_dgram.read_datagram().await {
                Ok(data) => {
                    tokio::spawn(handle_datagram(conn_dgram.clone(), data));
                }
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("Connection closed by application");
                    break;
                }
                Err(err) => {
                    error!("Failed to read datagram: {}", err);
                    break;
                }
            }
        }
    });

    // Wait for connection to close
    tokio::select! {
        _ = bidi_task => {},
        _ = uni_task => {},
        _ = dgram_task => {},
        _ = connection.closed() => {
            info!("Connection closed");
        }
    }
}
