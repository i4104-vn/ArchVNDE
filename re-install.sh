#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "============================================="
echo "   ArchVNDE Fresh Rebuild & Reinstall Script"
echo "============================================="

# 1. Clean previous build artifacts
echo "Cleaning previous cargo build artifacts..."
cargo clean

# 2. Check for cargo (Rust compiler)
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo (Rust toolchain) is not installed."
    echo "Please install Rust using: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 3. Check for pkg-config and gtk4 library headers
echo "Checking Arch Linux system dependencies..."
MISSING_DEPS=0

if ! pkg-config --exists gtk4; then
    echo "  - gtk4 dev headers are missing."
    MISSING_DEPS=1
fi

if ! pkg-config --exists gtk4-layer-shell; then
    echo "  - gtk4-layer-shell dev headers are missing."
    MISSING_DEPS=1
fi

if [ $MISSING_DEPS -eq 1 ]; then
    echo ""
    echo "Please install the missing build dependencies using pacman:"
    echo "    sudo pacman -S --needed pkgconf gtk4 gtk4-layer-shell"
    echo ""
    read -p "Would you like me to try installing them for you? (y/N) " install_choice
    if [[ "$install_choice" =~ ^[Yy]$ ]]; then
        sudo pacman -S --needed pkgconf gtk4 gtk4-layer-shell
    else
        echo "Please install them manually and re-run this script."
        exit 1
    fi
fi

# 4. Compile the workspace in release mode
echo "Rebuilding ArchVNDE components in release mode..."
cargo build --release

# 5. Create local bin directory if it doesn't exist
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"

# 6. Reinstall the binaries
echo "Overwriting binaries in $LOCAL_BIN..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-launcher "$LOCAL_BIN/archvnde-launcher"
cp target/release/archvnde-notification "$LOCAL_BIN/archvnde-notification"

echo "============================================="
echo "Reinstall complete!"
echo "Binaries updated at: $LOCAL_BIN"
echo "Default configuration is automatically created at ~/.config/archvnde/style.css on startup."
echo "============================================="
