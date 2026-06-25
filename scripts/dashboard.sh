#!/bin/bash
# SiHankor Dashboard Launcher
# Starts the dashboard server in background, keeps it alive across terminal sessions.
# Usage: ./scripts/dashboard.sh

cd "$(dirname "$0")/.."

# Kill any existing dashboard first
pkill -f sihankor-dashboard 2>/dev/null

# Build and start in background
cargo build --release --bin sihankor-dashboard 2>/dev/null
nohup target/release/sihankor-dashboard > /tmp/sihankor-dashboard.log 2>&1 &

echo "SiHankor Dashboard: http://localhost:9741"
echo "Logs: /tmp/sihankor-dashboard.log"
echo "Stop: pkill -f sihankor-dashboard"
