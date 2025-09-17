#!/bin/bash

# FlowDNS Development Environment Setup Script
# This script sets up a development environment for FlowDNS

set -e

# Colors
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

# Check if .env exists
check_env() {
    if [ ! -f .env ]; then
        print_info "Creating .env file..."
        cat > .env << 'EOF'
# FlowDNS Development Configuration
DATABASE_URL=postgresql://flowdns:devpassword@localhost/flowdns_dev
RUST_LOG=flowdns=debug,tower_http=debug
RUST_BACKTRACE=1
EOF
        print_success ".env file created"
    else
        print_success ".env file exists"
    fi
}

# Setup development database
setup_dev_db() {
    print_info "Setting up development database..."

    # Check if PostgreSQL is running
    if ! systemctl is-active --quiet postgresql; then
        print_info "Starting PostgreSQL..."
        sudo systemctl start postgresql
    fi

    # Create development database
    sudo -u postgres psql << EOF 2>/dev/null || true
DROP DATABASE IF EXISTS flowdns_dev;
CREATE DATABASE flowdns_dev;
DROP USER IF EXISTS flowdns;
CREATE USER flowdns WITH ENCRYPTED PASSWORD 'devpassword';
GRANT ALL PRIVILEGES ON DATABASE flowdns_dev TO flowdns;
ALTER DATABASE flowdns_dev OWNER TO flowdns;
\q
EOF

    print_success "Development database created"
}

# Install development tools
install_dev_tools() {
    print_info "Installing Rust development tools..."

    source "$HOME/.cargo/env"

    # Install useful cargo extensions
    cargo install cargo-watch 2>/dev/null || true
    cargo install cargo-edit 2>/dev/null || true
    cargo install sqlx-cli --no-default-features --features postgres 2>/dev/null || true

    print_success "Development tools installed"
}

# Run migrations
run_migrations() {
    print_info "Running database migrations..."

    source .env

    # Create migrations directory if it doesn't exist
    mkdir -p migrations

    # Run migrations using sqlx if available
    if command -v sqlx &> /dev/null; then
        sqlx migrate run
    else
        # Use psql directly
        if [ -f migrations/001_initial_schema.sql ]; then
            PGPASSWORD=devpassword psql -h localhost -U flowdns -d flowdns_dev -f migrations/001_initial_schema.sql
        fi
    fi

    print_success "Migrations completed"
}

# Insert test data
insert_test_data() {
    print_info "Inserting test data..."

    PGPASSWORD=devpassword psql -h localhost -U flowdns -d flowdns_dev << 'EOF'
-- Insert test subnet
INSERT INTO dhcp_subnets (
    name, network, start_ip, end_ip, gateway,
    dns_servers, domain_name, lease_duration, enabled, description
) VALUES (
    'test-subnet',
    '192.168.100.0/24',
    '192.168.100.10',
    '192.168.100.250',
    '192.168.100.1',
    '["8.8.8.8", "8.8.4.4"]'::jsonb,
    'test.local',
    3600,
    true,
    'Test subnet for development'
) ON CONFLICT (name) DO NOTHING;

-- Insert test reservation
INSERT INTO dhcp_reservations (
    subnet_id, mac_address, ip_address, hostname, description
) VALUES (
    (SELECT id FROM dhcp_subnets WHERE name = 'test-subnet'),
    E'\\x001122334455',
    '192.168.100.5',
    'test-device',
    'Test reservation'
) ON CONFLICT (mac_address) DO NOTHING;
EOF

    print_success "Test data inserted"
}

# Create development config
create_dev_config() {
    print_info "Creating development configuration..."

    cat > config/dev.toml << 'EOF'
# FlowDNS Development Configuration

[server]
log_level = "debug"
threads = 2

[database]
url = "postgresql://flowdns:devpassword@localhost/flowdns_dev"
max_connections = 5
min_connections = 1
connect_timeout = 5
idle_timeout = 300

[dns]
enabled = false
bind_address = "127.0.0.1"
port = 5353  # Non-standard port for development
forward_servers = ["8.8.8.8", "8.8.4.4"]
domain_suffix = "dev.local"
dynamic_updates = true
hostname_template = "dev-{ip_dash}"
ttl_default = 300
cache_size = 100

[dhcp]
enabled = true
bind_address = "127.0.0.1"
port = 6767  # Non-standard port for development
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
cors_origins = ["http://localhost:3000", "http://localhost:5173"]
jwt_secret = "dev-secret-key-only-for-development-do-not-use-in-production"
jwt_expiry = 86400

[subnets.dev]
network = "192.168.100.0/24"
start_ip = "192.168.100.10"
end_ip = "192.168.100.250"
gateway = "192.168.100.1"
dns_servers = ["192.168.100.1", "8.8.8.8"]
domain_name = "dev.local"
lease_time = 3600
description = "Development subnet"
enabled = true
EOF

    print_success "Development configuration created"
}

# Setup Git hooks
setup_git_hooks() {
    print_info "Setting up Git hooks..."

    # Create pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Run tests before committing

source "$HOME/.cargo/env"

echo "Running tests..."
cargo test --quiet

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Checking formatting..."
cargo fmt -- --check
EOF

    chmod +x .git/hooks/pre-commit
    print_success "Git hooks configured"
}

# Create Makefile
create_makefile() {
    print_info "Creating Makefile..."

    cat > Makefile << 'EOF'
.PHONY: help build run test clean dev watch migrate

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in release mode
	cargo build --release

dev: ## Run in development mode
	RUST_LOG=debug cargo run -- --config config/dev.toml

watch: ## Run with auto-reload on changes
	cargo watch -x 'run -- --config config/dev.toml'

test: ## Run all tests
	cargo test

test-watch: ## Run tests with auto-reload
	cargo watch -x test

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

migrate: ## Run database migrations
	./target/debug/flowdns --migrate

db-reset: ## Reset development database
	sudo -u postgres psql -c "DROP DATABASE IF EXISTS flowdns_dev;"
	sudo -u postgres psql -c "CREATE DATABASE flowdns_dev;"
	sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE flowdns_dev TO flowdns;"
	$(MAKE) migrate

clippy: ## Run clippy linter
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

check: ## Run all checks (test, clippy, fmt)
	cargo test
	cargo clippy -- -D warnings
	cargo fmt -- --check

docs: ## Generate documentation
	cargo doc --open

install-deps: ## Install development dependencies
	cargo install cargo-watch cargo-edit sqlx-cli --locked
EOF

    print_success "Makefile created"
}

# Main function
main() {
    echo "================================================"
    echo "      FlowDNS Development Environment Setup     "
    echo "================================================"
    echo ""

    cd "$(dirname "$0")/.."

    check_env
    setup_dev_db
    install_dev_tools
    run_migrations
    insert_test_data
    create_dev_config
    setup_git_hooks
    create_makefile

    echo ""
    print_success "Development environment setup complete!"
    echo ""
    print_info "Available commands:"
    echo "  make help      - Show all available commands"
    echo "  make dev       - Run in development mode"
    echo "  make watch     - Run with auto-reload"
    echo "  make test      - Run tests"
    echo "  make db-reset  - Reset development database"
    echo ""
    print_info "Development server will run on:"
    echo "  DHCP: 127.0.0.1:6767 (non-standard port)"
    echo "  DNS:  127.0.0.1:5353 (non-standard port)"
    echo "  API:  http://127.0.0.1:8080"
    echo ""
}

main "$@"