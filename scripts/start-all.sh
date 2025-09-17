#!/bin/bash

# Combined start script for FlowDNS backend and frontend

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()

    # Check for Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("Rust/Cargo")
    fi

    # Check for Node.js/npm
    if ! command -v node &> /dev/null; then
        missing_deps+=("Node.js")
    fi

    if ! command -v npm &> /dev/null; then
        missing_deps+=("npm")
    fi

    # Check for PostgreSQL client (optional but recommended)
    if ! command -v psql &> /dev/null; then
        print_warning "PostgreSQL client not found (optional)"
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        echo "Please install the missing dependencies and try again"
        exit 1
    fi

    print_status "All required dependencies found"
}

# Function to start all services
start_all() {
    echo "========================================="
    echo "Starting FlowDNS Services"
    echo "========================================="
    echo

    # Check dependencies
    check_dependencies

    # Build backend if needed
    if [ ! -f "$PROJECT_ROOT/target/release/flowdns" ] || [ "$1" = "--build" ]; then
        print_status "Building backend..."
        "$SCRIPT_DIR/build-backend.sh"
        echo
    fi

    # Start backend
    print_status "Starting backend service..."
    "$SCRIPT_DIR/start-backend.sh" start
    echo

    # Wait for backend to be ready
    print_status "Waiting for backend to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:8080/api/v1/system/health > /dev/null 2>&1; then
            print_status "Backend is ready!"
            break
        fi
        if [ $i -eq 30 ]; then
            print_error "Backend failed to start. Check logs at: $PROJECT_ROOT/logs/backend.log"
            exit 1
        fi
        sleep 1
    done
    echo

    # Start frontend
    print_status "Starting frontend service..."
    if [ "$2" = "--dev" ]; then
        "$SCRIPT_DIR/start-frontend.sh" dev
    else
        "$SCRIPT_DIR/start-frontend.sh" start
    fi
    echo

    echo "========================================="
    echo -e "${GREEN}All services started successfully!${NC}"
    echo "========================================="
    echo
    echo "Backend API: http://localhost:8080"
    echo "API Docs:    http://localhost:8080/api/v1/docs"
    echo "Frontend:    http://localhost:3000"
    echo
    echo "Logs:"
    echo "  Backend:  $PROJECT_ROOT/logs/backend.log"
    echo "  Frontend: $PROJECT_ROOT/logs/frontend.log"
    echo
}

# Function to stop all services
stop_all() {
    echo "========================================="
    echo "Stopping FlowDNS Services"
    echo "========================================="
    echo

    print_status "Stopping frontend..."
    "$SCRIPT_DIR/start-frontend.sh" stop
    echo

    print_status "Stopping backend..."
    "$SCRIPT_DIR/start-backend.sh" stop
    echo

    echo "========================================="
    echo -e "${GREEN}All services stopped${NC}"
    echo "========================================="
}

# Function to restart all services
restart_all() {
    stop_all
    echo
    sleep 2
    start_all "$@"
}

# Function to show status
show_status() {
    echo "========================================="
    echo "FlowDNS Services Status"
    echo "========================================="
    echo

    echo "Backend:"
    "$SCRIPT_DIR/start-backend.sh" status
    echo

    echo "Frontend:"
    "$SCRIPT_DIR/start-frontend.sh" status
    echo

    # Check if services are accessible
    if curl -s http://localhost:8080/api/v1/system/health > /dev/null 2>&1; then
        print_status "Backend API is accessible"
    else
        print_warning "Backend API is not accessible"
    fi

    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        print_status "Frontend is accessible"
    else
        print_warning "Frontend is not accessible"
    fi
}

# Function to tail logs
tail_logs() {
    echo "========================================="
    echo "Tailing FlowDNS Logs (Ctrl+C to stop)"
    echo "========================================="
    echo

    # Use tail with -F to follow logs even if they're rotated
    tail -F "$PROJECT_ROOT/logs/backend.log" "$PROJECT_ROOT/logs/frontend.log" 2>/dev/null
}

# Main logic
case "${1:-start}" in
    start)
        start_all "$2" "$3"
        ;;
    stop)
        stop_all
        ;;
    restart)
        restart_all "$2" "$3"
        ;;
    status)
        show_status
        ;;
    logs)
        tail_logs
        ;;
    build)
        "$SCRIPT_DIR/build-backend.sh" "$2"
        ;;
    dev)
        start_all "--build" "--dev"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs|build|dev} [options]"
        echo
        echo "Commands:"
        echo "  start   - Start all services"
        echo "  stop    - Stop all services"
        echo "  restart - Restart all services"
        echo "  status  - Show services status"
        echo "  logs    - Tail all service logs"
        echo "  build   - Build the backend"
        echo "  dev     - Start all services in development mode"
        echo
        echo "Options:"
        echo "  --build - Build backend before starting (with start/restart)"
        echo "  --dev   - Start frontend in development mode (with start/restart)"
        echo
        echo "Examples:"
        echo "  $0 start              # Start all services"
        echo "  $0 start --build      # Build backend and start all services"
        echo "  $0 start --build --dev # Build backend and start in dev mode"
        echo "  $0 dev                # Start everything in development mode"
        exit 1
        ;;
esac