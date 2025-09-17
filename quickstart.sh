#!/bin/bash

# FlowDNS Quick Start Script
# This script provides a minimal setup to get FlowDNS running quickly

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${YELLOW}➜${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

echo "================================================"
echo "         FlowDNS Quick Start Setup              "
echo "================================================"
echo ""

# Check if Rust is installed
if command -v cargo &> /dev/null; then
    print_success "Rust is installed: $(rustc --version)"
else
    print_error "Rust is not installed"
    print_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check for build tools
if ! command -v gcc &> /dev/null; then
    print_error "GCC not found - development tools needed"
    print_info "Please run: ./scripts/install-build-tools.sh"
    print_info "Or install: build-essential (Ubuntu) / base-devel (Arch) / Development Tools (Fedora)"
    exit 1
fi

# Create a simple SQLite configuration for testing
print_info "Creating test configuration..."

cat > config/test.toml << 'EOF'
# FlowDNS Test Configuration (SQLite - No PostgreSQL needed)

[server]
log_level = "debug"
threads = 2

[database]
# Using SQLite for testing - no PostgreSQL needed!
url = "sqlite:flowdns.db"
max_connections = 5
min_connections = 1
connect_timeout = 5
idle_timeout = 300

[dns]
enabled = false
bind_address = "127.0.0.1"
port = 5353
forward_servers = ["8.8.8.8"]
domain_suffix = "test.local"
dynamic_updates = true
hostname_template = "host-{ip_dash}"
ttl_default = 300
cache_size = 100

[dhcp]
enabled = true
bind_address = "127.0.0.1"
port = 6767  # Non-privileged port for testing
default_lease_time = 3600
max_lease_time = 7200
renewal_time = 1800
rebind_time = 3150
decline_time = 300

[ipv6]
enabled = false
radvd_config_path = "/tmp/radvd.conf"
prefix_length = 64
router_lifetime = 1800
reachable_time = 0
retransmit_time = 0

[routing]
management_subnet = "127.0.0.1/32"
upstream_gateway = "127.0.0.1"
enable_inter_subnet_routing = false
nat_enabled = false

[api]
enabled = true
bind_address = "127.0.0.1"
port = 8080
cors_enabled = true
cors_origins = ["http://localhost:3000"]
jwt_secret = "test-secret-key-at-least-32-characters-long"
jwt_expiry = 86400

[subnets.test]
network = "192.168.200.0/24"
start_ip = "192.168.200.10"
end_ip = "192.168.200.250"
gateway = "192.168.200.1"
dns_servers = ["8.8.8.8", "8.8.4.4"]
domain_name = "test.local"
lease_time = 3600
description = "Test subnet"
enabled = true
EOF

print_success "Test configuration created"

# Build the project
print_info "Building FlowDNS (this may take a few minutes)..."
export PATH="$HOME/.cargo/bin:$PATH"

if cargo build 2>&1; then
    print_success "Build successful!"
else
    print_error "Build failed"
    print_info "Common issues:"
    echo "  1. Missing build tools: Run ./scripts/install-build-tools.sh"
    echo "  2. Missing OpenSSL: Install libssl-dev (Ubuntu) or openssl-devel (Fedora)"
    echo "  3. Missing pkg-config: Install pkg-config package"
    exit 1
fi

echo ""
echo "================================================"
print_success "Quick start setup complete!"
echo "================================================"
echo ""

print_info "To run FlowDNS in test mode:"
echo ""
echo "  cargo run -- --config config/test.toml"
echo ""
print_info "Or run with debug logging:"
echo ""
echo "  RUST_LOG=debug cargo run -- --config config/test.toml"
echo ""
print_info "The test server will run on:"
echo "  - DHCP: 127.0.0.1:6767 (non-privileged port)"
echo "  - API: http://127.0.0.1:8080"
echo ""
print_info "For production setup with PostgreSQL, run:"
echo "  ./install.sh"
echo ""