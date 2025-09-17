#!/bin/bash

# Install essential build tools for compiling Rust projects

set -e

echo "Installing essential build tools..."

# Check if we can use apt
if command -v apt-get &> /dev/null; then
    echo "Using apt-get to install build tools..."
    sudo apt-get update
    sudo apt-get install -y build-essential gcc g++ make pkg-config libssl-dev
elif command -v dnf &> /dev/null; then
    echo "Using dnf to install build tools..."
    sudo dnf groupinstall -y "Development Tools"
    sudo dnf install -y gcc gcc-c++ make pkgconfig openssl-devel
elif command -v yum &> /dev/null; then
    echo "Using yum to install build tools..."
    sudo yum groupinstall -y "Development Tools"
    sudo yum install -y gcc gcc-c++ make pkgconfig openssl-devel
elif command -v pacman &> /dev/null; then
    echo "Using pacman to install build tools..."
    sudo pacman -S --noconfirm base-devel gcc openssl pkg-config
else
    echo "No supported package manager found"
    echo "Please install the following manually:"
    echo "  - gcc/g++ compiler"
    echo "  - make"
    echo "  - pkg-config"
    echo "  - OpenSSL development libraries"
    exit 1
fi

echo "Build tools installed successfully!"

# Verify installations
echo ""
echo "Verifying installations:"
gcc --version | head -1 || echo "gcc not found"
make --version | head -1 || echo "make not found"
pkg-config --version || echo "pkg-config not found"

echo ""
echo "Build tools setup complete!"