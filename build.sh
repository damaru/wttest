#!/bin/bash

echo "Building WebTransport Rust project..."

# Build the workspace
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "To run the project:"
    echo "1. Start the server: cargo run -p server"
    echo "2. In another terminal, run the client: cargo run -p client"
    echo ""
    echo "Client options:"
    echo "  --demo bidirectional  # Test bidirectional streams"
    echo "  --demo unidirectional # Test unidirectional streams"
    echo "  --demo datagram       # Test datagrams"
    echo "  --demo all            # Run all demos (default)"
    echo "  --insecure            # Skip TLS verification for self-signed certs"
    echo "  --verbose             # Enable verbose logging"
else
    echo "❌ Build failed!"
    exit 1
fi
