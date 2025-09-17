#!/bin/bash

# FlowDNS Offline Installation Script
# Use this script when behind a restrictive proxy or without internet access

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
echo "      FlowDNS Offline Installation Script      "
echo "================================================"
echo ""

# Check if Rust is already installed
if command -v cargo &> /dev/null; then
    print_success "Rust already installed: $(rustc --version)"
else
    print_info "Rust not found"
    echo ""
    echo "To install Rust offline:"
    echo ""
    echo "1. Download rustup-init from another machine:"
    echo "   Linux: https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
    echo "   macOS: https://static.rust-lang.org/rustup/dist/x86_64-apple-darwin/rustup-init"
    echo ""
    echo "2. Copy the file to this machine and run:"
    echo "   chmod +x rustup-init"
    echo "   ./rustup-init -y --no-modify-path"
    echo "   source \$HOME/.cargo/env"
    echo ""
    echo "3. Then run this script again"
    echo ""

    # Check if rustup-init exists in current directory
    if [ -f ./rustup-init ]; then
        print_info "Found rustup-init in current directory"
        read -p "Install Rust using local rustup-init? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            chmod +x ./rustup-init
            ./rustup-init -y --no-modify-path
            source "$HOME/.cargo/env"
            print_success "Rust installed successfully"
        fi
    fi
fi

echo ""
print_info "For environments with proxy:"
echo ""
echo "Set proxy environment variables before running install.sh:"
echo "  export http_proxy=http://your-proxy:port"
echo "  export https_proxy=http://your-proxy:port"
echo "  export HTTP_PROXY=http://your-proxy:port"
echo "  export HTTPS_PROXY=http://your-proxy:port"
echo ""
echo "Then run: ./install.sh"
echo ""

print_info "Alternative: Use Docker"
echo ""
echo "If Docker is available, you can build and run FlowDNS in containers:"
echo "  cd docker"
echo "  docker-compose up -d"
echo ""