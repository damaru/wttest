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

### Start the browser-testable server:
```bash
cargo run -p server
```

Then open this page in Chrome or Edge:

```text
http://127.0.0.1:8080
```

Click **Connect**, then send a datagram or stream message. The default `ping` datagram should receive `pong`.

### Run the Rust HTTP/3 WebTransport client:
```bash
cargo run -p client
```

You can also point it at another endpoint:

```bash
cargo run -p client -- --url https://127.0.0.1:4433/ --demo datagram
```

## Dependencies

- `wtransport` - WebTransport over HTTP/3 implementation
- `axum` - HTTP server for the browser test page
- `tokio` - Async runtime
- `anyhow` - Error handling
- `tracing` - Logging

## Notes

This implementation uses self-signed certificates for demonstration purposes. In production, you should use proper certificates from a trusted CA.
