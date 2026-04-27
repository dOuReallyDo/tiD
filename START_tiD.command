#!/bin/bash
# tiD - CVM Pricing Cockpit launcher

# Change to the directory where this script lives
cd "$(dirname "$0")"

echo "================================"
echo "tiD - CVM Pricing Cockpit"
echo "================================"
echo

# Start the server in background
./tid serve &
SERVER_PID=$!

# Wait a moment for the server to start
sleep 2

# Open browser (macOS)
open http://127.0.0.1:5002

echo "Server running at http://127.0.0.1:5002"
echo "Press Ctrl+C to stop..."
echo

# Wait for Ctrl+C
wait $SERVER_PID