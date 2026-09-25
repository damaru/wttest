use anyhow::Result;
use bytes::Bytes;
use quinn::Connection;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Demonstrate bidirectional streaming
pub async fn demo_bidirectional(connection: Arc<Connection>) -> Result<()> {
    info!("=== Bidirectional Stream Demo ===");
    
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // Send a message
    let message = "Hello from bidirectional stream!";
    send.write_all(message.as_bytes()).await?;
    send.finish()?;
    
    info!("Sent: {}", message);
    
    // Read the response
    let response = recv.read_to_end(1024).await?;
    let response_str = String::from_utf8_lossy(&response);
    info!("Received: {}", response_str);
    
    Ok(())
}

/// Demonstrate unidirectional streaming
pub async fn demo_unidirectional(connection: Arc<Connection>) -> Result<()> {
    info!("=== Unidirectional Stream Demo ===");
    
    // Send a log message
    let mut send = connection.open_uni().await?;
    let log_message = "log:This is a log message from the client";
    send.write_all(log_message.as_bytes()).await?;
    send.finish()?;
    info!("Sent log message: {}", log_message);
    
    // Wait a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Send a command
    let mut send = connection.open_uni().await?;
    let command_message = "command:status";
    send.write_all(command_message.as_bytes()).await?;
    send.finish()?;
    info!("Sent command: {}", command_message);
    
    Ok(())
}

/// Demonstrate datagram communication
pub async fn demo_datagram(connection: Arc<Connection>) -> Result<()> {
    info!("=== Datagram Demo ===");
    
    // Send ping
    let ping_data = Bytes::from("ping");
    connection.send_datagram(ping_data.clone())?;
    info!("Sent ping datagram");
    
    // Wait for response
    tokio::time::timeout(Duration::from_secs(5), async {
        if let Ok(response) = connection.read_datagram().await {
            let response_str = String::from_utf8_lossy(&response);
            info!("Received datagram response: {}", response_str);
        }
    }).await.ok();
    
    // Send time request
    let time_data = Bytes::from("time");
    connection.send_datagram(time_data.clone())?;
    info!("Sent time request datagram");
    
    // Wait for response
    tokio::time::timeout(Duration::from_secs(5), async {
        if let Ok(response) = connection.read_datagram().await {
            let response_str = String::from_utf8_lossy(&response);
            info!("Received time response: {}", response_str);
        }
    }).await.ok();
    
    // Send echo message
    let echo_data = Bytes::from("Hello, WebTransport!");
    connection.send_datagram(echo_data.clone())?;
    info!("Sent echo datagram: {}", String::from_utf8_lossy(&echo_data));
    
    // Wait for echo response
    tokio::time::timeout(Duration::from_secs(5), async {
        if let Ok(response) = connection.read_datagram().await {
            let response_str = String::from_utf8_lossy(&response);
            info!("Received echo response: {}", response_str);
        }
    }).await.ok();
    
    Ok(())
}
