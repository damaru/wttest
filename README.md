# Rust WebTransport Client and Server

This project demonstrates a WebTransport implementation in Rust using QUIC as the underlying transport protocol.

## Project Structure

- `server/` - WebTransport server implementation
- `client/` - WebTransport client implementation

## Features

- Bidirectional streaming
- Unidirectional streaming
- Datagram support
- TLS 1.3 with self-signed certificates
- Async/await support with Tokio

## Building

```bash
cargo build
```

## Running

### Start the server:
```bash
cargo run -p server
```

### Run the client:
```bash
cargo run -p client
```

## Dependencies

- `quinn` - QUIC implementation
- `tokio` - Async runtime
- `rustls` - TLS implementation
- `rcgen` - Certificate generation
- `anyhow` - Error handling
- `tracing` - Logging

## Notes

This implementation uses self-signed certificates for demonstration purposes. In production, you should use proper certificates from a trusted CA.
