#!/bin/bash

# Build script for FlowDNS backend

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "========================================="
echo "Building FlowDNS Backend"
echo "========================================="

cd "$PROJECT_ROOT"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Clean previous build artifacts if requested
if [ "$1" = "--clean" ]; then
    echo "Cleaning previous build artifacts..."
    cargo clean
fi

# Build the project in release mode
echo "Building backend in release mode..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "========================================="
    echo "Build successful!"
    echo "Binary location: $PROJECT_ROOT/target/release/flowdns"
    echo "========================================="
else
    echo "========================================="
    echo "Build failed!"
    echo "========================================="
    exit 1
fi