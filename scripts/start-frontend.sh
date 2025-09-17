#!/bin/bash

# Start script for FlowDNS frontend with process management

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FRONTEND_DIR="$PROJECT_ROOT/web"
PID_FILE="$PROJECT_ROOT/.flowdns-frontend.pid"
LOG_FILE="$PROJECT_ROOT/logs/frontend.log"
PORT=${FRONTEND_PORT:-3000}

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

# Function to check if port is in use
is_port_in_use() {
    lsof -i:$PORT > /dev/null 2>&1
}

# Function to stop the frontend
stop_frontend() {
    if is_running; then
        PID=$(cat "$PID_FILE")
        echo "Stopping FlowDNS frontend (PID: $PID)..."

        # Kill the process group to ensure all child processes are terminated
        kill -TERM -$(ps -o pgid= $PID | grep -o '[0-9]*') 2>/dev/null || kill "$PID" 2>/dev/null || true

        # Wait for process to terminate
        for i in {1..10}; do
            if ! ps -p "$PID" > /dev/null 2>&1; then
                echo "Frontend stopped successfully"
                rm -f "$PID_FILE"
                return 0
            fi
            sleep 1
        done

        # Force kill if still running
        echo "Force stopping frontend..."
        kill -9 -$(ps -o pgid= $PID | grep -o '[0-9]*') 2>/dev/null || kill -9 "$PID" 2>/dev/null || true
        rm -f "$PID_FILE"
    else
        echo "Frontend is not running"
    fi

    # Also check if port is still in use
    if is_port_in_use; then
        echo "Port $PORT is still in use. Attempting to free it..."
        PID=$(lsof -t -i:$PORT)
        if [ ! -z "$PID" ]; then
            kill -9 $PID 2>/dev/null || true
        fi
    fi
}

# Function to start the frontend
start_frontend() {
    # Check if frontend directory exists
    if [ ! -d "$FRONTEND_DIR" ]; then
        echo "Error: Frontend directory not found at $FRONTEND_DIR"
        exit 1
    fi

    # Check if already running
    if is_running; then
        echo "Frontend is already running (PID: $(cat "$PID_FILE"))"
        echo "Restarting..."
        stop_frontend
    fi

    # Check if port is in use
    if is_port_in_use; then
        echo "Port $PORT is already in use"
        echo "Attempting to stop the process using it..."
        PID=$(lsof -t -i:$PORT)
        if [ ! -z "$PID" ]; then
            kill -9 $PID 2>/dev/null || true
            sleep 1
        fi
    fi

    cd "$FRONTEND_DIR"

    # Check if node_modules exists
    if [ ! -d "node_modules" ]; then
        echo "Installing frontend dependencies..."
        npm install
    fi

    # Check if it's a Next.js project
    if [ -f "next.config.js" ] || [ -f "next.config.mjs" ]; then
        echo "Detected Next.js project"

        # Build the frontend if not in development mode
        if [ "${NODE_ENV}" != "development" ] && [ "${1}" != "--dev" ]; then
            echo "Building frontend for production..."
            npm run build

            echo "Starting Next.js frontend in production mode on port $PORT..."
            nohup npm run start -- -p $PORT >> "$LOG_FILE" 2>&1 &
        else
            echo "Starting Next.js frontend in development mode on port $PORT..."
            nohup npm run dev -- -p $PORT >> "$LOG_FILE" 2>&1 &
        fi
    # Check if it's a Vite project
    elif [ -f "vite.config.ts" ] || [ -f "vite.config.js" ]; then
        echo "Detected Vite project"

        if [ "${NODE_ENV}" != "development" ] && [ "${1}" != "--dev" ]; then
            echo "Building frontend for production..."
            npm run build

            # Check if serve is installed globally
            if ! command -v serve &> /dev/null; then
                echo "Installing serve globally..."
                npm install -g serve
            fi

            echo "Starting Vite frontend in production mode on port $PORT..."
            nohup serve -s dist -l $PORT --no-clipboard >> "$LOG_FILE" 2>&1 &
        else
            echo "Starting Vite frontend in development mode on port $PORT..."
            nohup npm run dev -- --host 0.0.0.0 --port $PORT >> "$LOG_FILE" 2>&1 &
        fi
    # Check if it's a React project (non-Vite)
    elif [ -f "package.json" ] && grep -q "\"react\"" package.json && ! grep -q "vite" package.json; then
        echo "Detected React project (non-Vite)"

        if [ "${NODE_ENV}" != "development" ] && [ "${1}" != "--dev" ]; then
            echo "Building frontend for production..."
            npm run build

            # Check if serve is installed globally
            if ! command -v serve &> /dev/null; then
                echo "Installing serve globally..."
                npm install -g serve
            fi

            echo "Starting React frontend in production mode on port $PORT..."
            nohup serve -s build -l $PORT >> "$LOG_FILE" 2>&1 &
        else
            echo "Starting React frontend in development mode on port $PORT..."
            PORT=$PORT nohup npm start >> "$LOG_FILE" 2>&1 &
        fi
    else
        # Generic npm start
        echo "Starting frontend on port $PORT..."
        PORT=$PORT nohup npm start >> "$LOG_FILE" 2>&1 &
    fi

    PID=$!

    # Save PID
    echo $PID > "$PID_FILE"

    # Wait a moment to check if it started successfully
    sleep 3

    if is_running; then
        echo "========================================="
        echo "Frontend started successfully"
        echo "PID: $PID"
        echo "URL: http://localhost:$PORT"
        echo "Log file: $LOG_FILE"
        echo "========================================="
    else
        echo "Failed to start frontend. Check logs at: $LOG_FILE"
        exit 1
    fi
}

# Main logic
case "${1:-start}" in
    start)
        start_frontend "$2"
        ;;
    stop)
        stop_frontend
        ;;
    restart)
        stop_frontend
        sleep 1
        start_frontend "$2"
        ;;
    status)
        if is_running; then
            echo "Frontend is running (PID: $(cat "$PID_FILE"))"
            echo "URL: http://localhost:$PORT"
        else
            echo "Frontend is not running"
        fi
        ;;
    dev)
        start_frontend "--dev"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|dev}"
        echo "  start   - Start in production mode"
        echo "  stop    - Stop the frontend"
        echo "  restart - Restart the frontend"
        echo "  status  - Check if frontend is running"
        echo "  dev     - Start in development mode"
        exit 1
        ;;
esac