#!/bin/bash

# FlowDNS Installation Script
# This script installs all dependencies and sets up FlowDNS

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RUST_INSTALL_TIMEOUT=30
SKIP_RUST_INSTALL=false
USE_EXISTING_RUST=false

# Functions
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}➜${NC} $1"
}

check_command() {
    if command -v $1 &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Check for proxy settings
detect_proxy() {
    if [ -n "$HTTP_PROXY" ] || [ -n "$http_proxy" ] || [ -n "$HTTPS_PROXY" ] || [ -n "$https_proxy" ]; then
        print_info "Proxy detected:"
        [ -n "$HTTP_PROXY" ] && echo "  HTTP_PROXY=$HTTP_PROXY"
        [ -n "$http_proxy" ] && echo "  http_proxy=$http_proxy"
        [ -n "$HTTPS_PROXY" ] && echo "  HTTPS_PROXY=$HTTPS_PROXY"
        [ -n "$https_proxy" ] && echo "  https_proxy=$https_proxy"
        echo ""

        # Export for curl
        export http_proxy="${http_proxy:-$HTTP_PROXY}"
        export https_proxy="${https_proxy:-$HTTPS_PROXY}"

        print_info "Proxy settings will be used for downloads"
    fi
}

# Detect OS
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$NAME
        VER=$VERSION_ID
    else
        print_error "Cannot detect operating system"
        exit 1
    fi
}

# Install Rust with better error handling
install_rust() {
    print_info "Checking Rust installation..."

    if check_command rustc && check_command cargo; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust already installed: $RUST_VERSION"
        USE_EXISTING_RUST=true
        return 0
    fi

    # Check if user wants to skip Rust installation
    read -t 10 -p "Rust not found. Install Rust now? (y/n, default: y): " -n 1 -r install_rust_answer || install_rust_answer="y"
    echo

    if [[ ! $install_rust_answer =~ ^[Yy]$ ]] && [ -n "$install_rust_answer" ]; then
        print_info "Skipping Rust installation"
        print_info "You can install Rust later by running:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        SKIP_RUST_INSTALL=true
        return 1
    fi

    print_info "Installing Rust (this may take a few minutes)..."

    # Create a temporary file for the installer
    RUST_INSTALLER=$(mktemp /tmp/rustup-init.XXXXXX)

    # Download rustup-init with timeout and proxy support
    print_info "Downloading Rust installer..."
    if curl --proto '=https' --tlsv1.2 -sSf \
            --connect-timeout 10 \
            --max-time 60 \
            -o "$RUST_INSTALLER" \
            https://sh.rustup.rs; then

        print_success "Rust installer downloaded"

        # Make it executable
        chmod +x "$RUST_INSTALLER"

        # Run the installer with automatic yes
        print_info "Running Rust installer..."
        if "$RUST_INSTALLER" -y --no-modify-path; then
            print_success "Rust installed successfully"

            # Add Rust to PATH for this session
            export PATH="$HOME/.cargo/bin:$PATH"
            source "$HOME/.cargo/env" 2>/dev/null || true

            # Clean up
            rm -f "$RUST_INSTALLER"

            return 0
        else
            print_error "Rust installation failed"
            rm -f "$RUST_INSTALLER"
            return 1
        fi
    else
        print_error "Failed to download Rust installer"
        print_info "Possible issues:"
        echo "  - Network connectivity problems"
        echo "  - Proxy configuration needed"
        echo "  - Firewall blocking HTTPS"
        echo ""
        print_info "To install with proxy:"
        echo "  export https_proxy=http://your-proxy:port"
        echo "  export http_proxy=http://your-proxy:port"
        echo "  ./install.sh"
        echo ""
        print_info "Or download installer manually from: https://rustup.rs"

        rm -f "$RUST_INSTALLER"
        SKIP_RUST_INSTALL=true
        return 1
    fi
}

# Install PostgreSQL
install_postgresql() {
    print_info "Checking PostgreSQL installation..."

    if check_command psql; then
        PG_VERSION=$(psql --version)
        print_success "PostgreSQL already installed: $PG_VERSION"
    else
        print_info "PostgreSQL not found"

        read -t 10 -p "Install PostgreSQL? (y/n, default: n): " -n 1 -r install_pg_answer || install_pg_answer="n"
        echo

        if [[ $install_pg_answer =~ ^[Yy]$ ]]; then
            print_info "Installing PostgreSQL..."

            case "$OS" in
                "Ubuntu"|"Debian GNU/Linux")
                    sudo apt-get update
                    sudo apt-get install -y postgresql postgresql-contrib libpq-dev
                    ;;
                "Fedora"|"CentOS Linux"|"Red Hat Enterprise Linux")
                    sudo dnf install -y postgresql postgresql-server postgresql-contrib postgresql-devel
                    sudo postgresql-setup --initdb
                    ;;
                "Arch Linux")
                    sudo pacman -S --noconfirm postgresql
                    sudo -u postgres initdb -D /var/lib/postgres/data
                    ;;
                *)
                    print_error "Unsupported OS for automatic PostgreSQL installation"
                    print_info "Please install PostgreSQL manually"
                    return 1
                    ;;
            esac

            # Start PostgreSQL service
            sudo systemctl enable postgresql 2>/dev/null || true
            sudo systemctl start postgresql 2>/dev/null || true
            print_success "PostgreSQL installed and started"
        else
            print_info "Skipping PostgreSQL installation"
            print_info "FlowDNS can use SQLite for testing without PostgreSQL"
        fi
    fi
}

# Install system dependencies with better error handling
install_system_deps() {
    print_info "Installing system dependencies..."

    # Check if build tools are already installed
    if check_command gcc && check_command make && check_command pkg-config; then
        print_success "Build tools already installed"
        return 0
    fi

    case "$OS" in
        "Ubuntu"|"Debian GNU/Linux")
            print_info "Using apt-get to install dependencies..."
            if sudo apt-get update; then
                sudo apt-get install -y \
                    build-essential \
                    pkg-config \
                    libssl-dev \
                    git \
                    curl \
                    net-tools \
                    iproute2 || {
                    print_error "Some packages failed to install"
                    print_info "You may need to install them manually"
                }
            else
                print_error "apt-get update failed - check your network connection"
                return 1
            fi
            ;;
        "Fedora"|"CentOS Linux"|"Red Hat Enterprise Linux")
            sudo dnf groupinstall -y "Development Tools" || true
            sudo dnf install -y \
                openssl-devel \
                pkg-config \
                git \
                curl \
                net-tools \
                iproute || true
            ;;
        "Arch Linux")
            sudo pacman -S --noconfirm \
                base-devel \
                openssl \
                pkg-config \
                git \
                curl \
                net-tools \
                iproute2 || true
            ;;
        *)
            print_error "Unsupported OS for automatic dependency installation"
            print_info "Please install: gcc, make, pkg-config, openssl-dev"
            return 1
            ;;
    esac

    print_success "System dependencies installed"
}

# Setup PostgreSQL database
setup_database() {
    if ! check_command psql; then
        print_info "PostgreSQL not installed, skipping database setup"
        print_info "You can use SQLite for testing (see quickstart.sh)"
        return 0
    fi

    print_info "Setting up PostgreSQL database..."

    # Check if database exists
    if sudo -u postgres psql -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw flowdns; then
        print_success "Database 'flowdns' already exists"
    else
        print_info "Creating database and user..."

        # Generate a random password
        DB_PASSWORD=$(openssl rand -base64 32 2>/dev/null || echo "flowdns_dev_password")

        # Create database and user
        if ! sudo -u postgres psql << EOF 2>/dev/null
CREATE DATABASE flowdns;
CREATE USER flowdns WITH ENCRYPTED PASSWORD '$DB_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE flowdns TO flowdns;
ALTER DATABASE flowdns OWNER TO flowdns;
\q
EOF
        then
            print_error "Failed to create database"
            print_info "You may need to create it manually"
            return 1
        fi

        print_success "Database created successfully"
        print_info "Database credentials:"
        echo "  Database: flowdns"
        echo "  User: flowdns"
        echo "  Password: $DB_PASSWORD"
        echo ""
        print_info "Please save these credentials and update config/server.toml"

        # Save credentials to a file
        cat > .env << EOF
# FlowDNS Database Configuration
DATABASE_URL=postgresql://flowdns:$DB_PASSWORD@localhost/flowdns
DB_USER=flowdns
DB_PASSWORD=$DB_PASSWORD
DB_NAME=flowdns
EOF
        chmod 600 .env
        print_success "Credentials saved to .env file (protected)"
    fi
}

# Build FlowDNS
build_flowdns() {
    if [ "$SKIP_RUST_INSTALL" = true ]; then
        print_info "Skipping build (Rust not installed)"
        return 0
    fi

    print_info "Building FlowDNS..."

    # Add Rust to PATH
    export PATH="$HOME/.cargo/bin:$PATH"
    source "$HOME/.cargo/env" 2>/dev/null || true

    # Check if cargo is available
    if ! check_command cargo; then
        print_error "Cargo not found in PATH"
        print_info "Please ensure Rust is installed and run:"
        echo "  source \$HOME/.cargo/env"
        echo "  cargo build --release"
        return 1
    fi

    # Build the project
    print_info "This may take several minutes on first build..."
    if cargo build --release 2>&1 | tee build.log; then
        print_success "FlowDNS built successfully"
        print_info "Binary location: target/release/flowdns"
        rm -f build.log
    else
        print_error "Build failed - check build.log for details"
        print_info "Common issues:"
        echo "  1. Missing GCC: sudo apt-get install build-essential"
        echo "  2. Missing OpenSSL: sudo apt-get install libssl-dev"
        echo "  3. Missing pkg-config: sudo apt-get install pkg-config"
        return 1
    fi
}

# Run migrations
run_migrations() {
    print_info "Checking for migrations..."

    # Check if .env exists
    if [ ! -f .env ]; then
        print_info "No .env file found, skipping migrations"
        return 0
    fi

    # Source environment variables
    source .env

    # Update config file with database URL if it exists
    if [ -f config/server.toml ] && [ -n "$DATABASE_URL" ]; then
        # Backup original config
        cp config/server.toml config/server.toml.bak 2>/dev/null || true

        # Update database URL in config
        sed -i "s|url = \".*\"|url = \"$DATABASE_URL\"|" config/server.toml 2>/dev/null || true
        print_success "Configuration updated with database credentials"
    fi

    # Run migrations using the built binary
    if [ -f target/release/flowdns ]; then
        print_info "Running database migrations..."
        ./target/release/flowdns --migrate 2>/dev/null || {
            print_info "Migrations skipped or already applied"
        }
    fi
}

# Create systemd service
create_service() {
    print_info "Creating systemd service..."

    INSTALL_DIR=$(pwd)
    SERVICE_FILE="/etc/systemd/system/flowdns.service"

    sudo tee $SERVICE_FILE > /dev/null << EOF
[Unit]
Description=FlowDNS DNS/DHCP Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/target/release/flowdns
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=flowdns

# Security settings
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=$INSTALL_DIR
ProtectHome=true
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    print_success "Systemd service created"
    print_info "To start FlowDNS: sudo systemctl start flowdns"
    print_info "To enable at boot: sudo systemctl enable flowdns"
}

# Main installation flow
main() {
    echo "================================================"
    echo "         FlowDNS Installation Script            "
    echo "================================================"
    echo ""

    # Check if running as root for system packages
    if [ "$EUID" -eq 0 ]; then
        print_error "Please don't run this script as root"
        print_info "The script will use sudo when needed"
        exit 1
    fi

    # Detect proxy settings
    detect_proxy

    # Detect OS
    detect_os
    print_success "Detected OS: $OS $VER"

    # Install dependencies
    install_rust || {
        print_info "Continuing without Rust..."
    }

    install_system_deps || {
        print_info "Some system dependencies may be missing"
    }

    install_postgresql || {
        print_info "Continuing without PostgreSQL..."
    }

    # Setup database
    setup_database || {
        print_info "Database setup skipped"
    }

    # Build project
    build_flowdns || {
        print_info "Build skipped or failed"
    }

    # Run migrations
    run_migrations || {
        print_info "Migrations skipped"
    }

    # Create service
    if [ -f target/release/flowdns ]; then
        read -t 10 -p "Create systemd service? (y/n, default: n): " -n 1 -r service_answer || service_answer="n"
        echo
        if [[ $service_answer =~ ^[Yy]$ ]]; then
            create_service
        fi
    fi

    echo ""
    echo "================================================"
    print_success "Installation completed!"
    echo "================================================"
    echo ""

    if [ "$SKIP_RUST_INSTALL" = true ]; then
        print_info "Note: Rust was not installed"
        echo "  To complete setup, install Rust:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
    fi

    print_info "Next steps:"

    if [ -f target/release/flowdns ]; then
        echo "  1. Review and update config/server.toml"
        echo "  2. Run migrations: ./target/release/flowdns --migrate"
        echo "  3. Start the server: sudo ./target/release/flowdns"
        echo "  4. Or use systemd: sudo systemctl start flowdns"
    else
        echo "  1. Install missing dependencies"
        echo "  2. Build the project: cargo build --release"
        echo "  3. Configure and run FlowDNS"
    fi

    echo ""
    print_info "For testing without PostgreSQL, use:"
    echo "  ./quickstart.sh"
    echo ""
}

# Run main function
main "$@"