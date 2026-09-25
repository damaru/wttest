use anyhow::Result;
use std::time::Duration;
use tracing::info;
use wtransport::Connection;

/// Demonstrate bidirectional streaming
pub async fn demo_bidirectional(connection: &Connection) -> Result<()> {
    info!("=== Bidirectional Stream Demo ===");

    let (mut send, mut recv) = connection.open_bi().await?.await?;

    let message = "Hello from bidirectional stream!";
    send.write_all(message.as_bytes()).await?;
    send.finish().await?;

    info!("Sent: {}", message);

    let response = read_stream_to_string(&mut recv).await?;
    info!("Received: {}", response);

    Ok(())
}

/// Demonstrate unidirectional streaming
pub async fn demo_unidirectional(connection: &Connection) -> Result<()> {
    info!("=== Unidirectional Stream Demo ===");

    let mut send = connection.open_uni().await?.await?;
    let message = "log:This is a log message from the client";
    send.write_all(message.as_bytes()).await?;
    send.finish().await?;
    info!("Sent unidirectional message: {}", message);

    let mut response = connection.accept_uni().await?;
    let response = read_stream_to_string(&mut response).await?;
    info!("Received server unidirectional response: {}", response);

    Ok(())
}

/// Demonstrate datagram communication
pub async fn demo_datagram(connection: &Connection) -> Result<()> {
    info!("=== Datagram Demo ===");

    send_datagram_and_log_response(connection, "ping").await?;
    send_datagram_and_log_response(connection, "Hello, WebTransport over HTTP/3!").await?;

    Ok(())
}

async fn send_datagram_and_log_response(connection: &Connection, message: &str) -> Result<()> {
    connection.send_datagram(message.as_bytes())?;
    info!("Sent datagram: {}", message);

    let response =
        tokio::time::timeout(Duration::from_secs(5), connection.receive_datagram()).await??;
    let response = String::from_utf8_lossy(&response);
    info!("Received datagram response: {}", response);

    Ok(())
}

async fn read_stream_to_string(recv: &mut wtransport::stream::RecvStream) -> Result<String> {
    let mut buffer = vec![0; 4096];
    let mut data = Vec::new();

    while let Some(bytes_read) = recv.read(&mut buffer).await? {
        data.extend_from_slice(&buffer[..bytes_read]);
    }

    Ok(String::from_utf8_lossy(&data).to_string())
}
