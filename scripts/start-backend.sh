#!/bin/bash

# Start script for FlowDNS backend with process management

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PID_FILE="$PROJECT_ROOT/.flowdns-backend.pid"
LOG_FILE="$PROJECT_ROOT/logs/backend.log"
BINARY="$PROJECT_ROOT/target/release/flowdns"

# Create logs directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/logs"

# Function to check if process is running
is_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        else
            # PID file exists but process is not running
            rm -f "$PID_FILE"
        fi
    fi
    return 1
}

# Function to stop the backend
stop_backend() {
    if is_running; then
        PID=$(cat "$PID_FILE")
        echo "Stopping FlowDNS backend (PID: $PID)..."
        kill "$PID" 2>/dev/null || true

        # Wait for process to terminate
        for i in {1..10}; do
            if ! ps -p "$PID" > /dev/null 2>&1; then
                echo "Backend stopped successfully"
                rm -f "$PID_FILE"
                return 0
            fi
            sleep 1
        done

        # Force kill if still running
        echo "Force stopping backend..."
        kill -9 "$PID" 2>/dev/null || true
        rm -f "$PID_FILE"
    else
        echo "Backend is not running"
    fi
}

# Function to start the backend
start_backend() {
    # Check if binary exists
    if [ ! -f "$BINARY" ]; then
        echo "Error: Backend binary not found at $BINARY"
        echo "Please run ./scripts/build-backend.sh first"
        exit 1
    fi

    # Check if already running
    if is_running; then
        echo "Backend is already running (PID: $(cat "$PID_FILE"))"
        echo "Restarting..."
        stop_backend
    fi

    # Load environment variables
    if [ -f "$PROJECT_ROOT/.env" ]; then
        export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)
    fi

    echo "Starting FlowDNS backend..."
    cd "$PROJECT_ROOT"

    # Check if port is already in use
    if lsof -i:8080 >/dev/null 2>&1; then
        echo "Port 8080 is already in use. Checking for stale processes..."
        EXISTING_PID=$(lsof -t -i:8080)
        if [ ! -z "$EXISTING_PID" ]; then
            echo "Found process $EXISTING_PID using port 8080. Killing it..."
            kill -9 $EXISTING_PID 2>/dev/null || true
            sleep 1
        fi
    fi

    # Start the backend in the background
    nohup "$BINARY" >> "$LOG_FILE" 2>&1 &
    PID=$!

    # Save PID
    echo $PID > "$PID_FILE"

    # Wait a moment to check if it started successfully
    sleep 2

    if is_running; then
        echo "========================================="
        echo "Backend started successfully"
        echo "PID: $PID"
        echo "Log file: $LOG_FILE"
        echo "========================================="
    else
        echo "Failed to start backend. Check logs at: $LOG_FILE"
        exit 1
    fi
}

# Main logic
case "${1:-start}" in
    start)
        start_backend
        ;;
    stop)
        stop_backend
        ;;
    restart)
        stop_backend
        sleep 1
        start_backend
        ;;
    status)
        if is_running; then
            echo "Backend is running (PID: $(cat "$PID_FILE"))"
        else
            echo "Backend is not running"
        fi
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac