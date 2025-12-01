#!/bin/bash
# Start dev servers for Tauri development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Check if API is already running on port 9876
if ! curl -s http://localhost:9876/health > /dev/null 2>&1; then
    echo "Starting API server on port 9876..."
    cd "$PROJECT_ROOT/api"
    PORT=9876 cargo run &
    API_PID=$!

    # Wait for API to be ready (max 60 seconds)
    echo "Waiting for API server to start..."
    for i in {1..60}; do
        if curl -s http://localhost:9876/health > /dev/null 2>&1; then
            echo "API server is ready!"
            break
        fi
        sleep 1
    done
else
    echo "API server already running on port 9876"
fi

# Check if webui is already running on port 3000
if ! curl -s http://localhost:3000 > /dev/null 2>&1; then
    echo "Starting webui dev server..."
    cd "$PROJECT_ROOT/webui"
    npm run dev &
    WEBUI_PID=$!

    # Wait for webui to be ready (max 30 seconds)
    echo "Waiting for webui dev server to start..."
    for i in {1..30}; do
        if curl -s http://localhost:3000 > /dev/null 2>&1; then
            echo "Webui dev server is ready!"
            break
        fi
        sleep 1
    done
else
    echo "Webui already running on port 3000"
fi

echo "Dev servers started successfully"
