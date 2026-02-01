#!/bin/bash
# Start rhxd test server

cd "$(dirname "$0")"

# Create necessary directories
mkdir -p files logs

# Start the server
./target/release/rhxd --config test-server.json serve
