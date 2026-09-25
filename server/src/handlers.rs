use bytes::Bytes;
use quinn::{Connection, RecvStream, SendStream};
use std::sync::Arc;
use tracing::{error, info};

/// Handle bidirectional streams
pub async fn handle_bidirectional_stream(mut send: SendStream, mut recv: RecvStream) {
    info!("New bidirectional stream");
    
    // Read data from the stream
    match recv.read_to_end(1024 * 1024).await {
        Ok(data) => {
            let message = String::from_utf8_lossy(&data);
            info!("Received message: {}", message);
            
            // Echo the message back
            let response = format!("Echo: {}", message);
            if let Err(err) = send.write_all(response.as_bytes()).await {
                error!("Failed to write response: {}", err);
                return;
            }
            
            if let Err(err) = send.finish() {
                error!("Failed to finish stream: {}", err);
            }
        }
        Err(err) => {
            error!("Failed to read from bidirectional stream: {}", err);
        }
    }
}

/// Handle unidirectional streams
pub async fn handle_unidirectional_stream(mut recv: RecvStream) {
    info!("New unidirectional stream");
    
    match recv.read_to_end(1024 * 1024).await {
        Ok(data) => {
            let message = String::from_utf8_lossy(&data);
            info!("Received unidirectional message: {}", message);
            
            // Process the message (in this case, just log it)
            if message.starts_with("log:") {
                info!("Log message: {}", &message[4..]);
            } else if message.starts_with("command:") {
                info!("Command received: {}", &message[8..]);
                // In a real application, you might execute the command
            } else {
                info!("Unknown message type: {}", message);
            }
        }
        Err(err) => {
            error!("Failed to read from unidirectional stream: {}", err);
        }
    }
}

/// Handle datagrams
pub async fn handle_datagram(connection: Arc<Connection>, data: Bytes) {
    let message = String::from_utf8_lossy(&data);
    info!("Received datagram: {}", message);
    
    // Process the datagram
    if message.starts_with("ping") {
        // Send pong response
        let response = "pong".as_bytes();
        if let Err(err) = connection.send_datagram(Bytes::from(response)) {
            error!("Failed to send pong response: {}", err);
        } else {
            info!("Sent pong response");
        }
    } else if message.starts_with("time") {
        // Send current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let response = format!("time:{}", now);
        if let Err(err) = connection.send_datagram(Bytes::from(response)) {
            error!("Failed to send time response: {}", err);
        } else {
            info!("Sent time response: {}", now);
        }
    } else {
        // Echo the datagram back
        if let Err(err) = connection.send_datagram(data) {
            error!("Failed to echo datagram: {}", err);
        } else {
            info!("Echoed datagram back");
        }
    }
}
