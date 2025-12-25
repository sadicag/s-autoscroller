#!/bin/bash

set -e  # Exit on error

echo "=== s-autoscroller installation script ==="

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Cargo not found. Installing Rust and Cargo..."
    
    # Install Rust using rustup (official method)
    if ! command -v rustup &> /dev/null; then
        echo "Downloading and installing rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source cargo environment
        source "$HOME/.cargo/env"
    fi
    
    echo "Rust and Cargo installed successfully!"
else
    echo "Cargo found: $(cargo --version)"
fi

# Verify cargo is now available
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo installation failed or not in PATH"
    echo "Please restart your terminal or run: source \$HOME/.cargo/env"
    exit 1
fi

# Build the project
echo "Building s-autoscroller in release mode..."
cargo build --release

# Copy binary to current directory
echo "Copying binary to current directory..."
cp target/release/s-autoscroller .

echo ""
echo "=== Installation complete! ==="
echo "Binary location: $(pwd)/s-autoscroller"
echo ""
echo "To run: ./s-autoscroller"
echo "To install system-wide (optional): sudo cp s-autoscroller /usr/local/bin/"
