#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "============================================="
echo "   ArchVNDE Fresh Rebuild & Reinstall Script"
echo "============================================="

# 1. Install/Update all dependencies and the Rust toolchain first via pacman
echo "Ensuring Arch Linux packages, development tools, and Rust compiler are installed..."
sudo pacman -S --needed --noconfirm base-devel git pkgconf gtk4 gtk4-layer-shell rust labwc

# 2. Clean previous build artifacts
echo "Cleaning previous cargo build cache..."
cargo clean

# 3. Compile the workspace in release mode
echo "Rebuilding ArchVNDE components in release mode..."
cargo build --release

# 4. Create local bin directory if it doesn't exist
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"

# 5. Kill any running instances first so the new binaries can be loaded
echo "Stopping any running shell processes..."
killall archvnde-panel || true
killall archvnde-launcher || true
killall archvnde-menu || true
killall archvnde-dock || true
killall archvnde-notification || true

# Remove old deprecated notification binary
rm -f "$LOCAL_BIN/archvnde-notification"

# 6. Reinstall the binaries
echo "Overwriting binaries in $LOCAL_BIN..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-launcher "$LOCAL_BIN/archvnde-launcher"
cp target/release/archvnde-menu "$LOCAL_BIN/archvnde-menu"
cp target/release/archvnde-dock "$LOCAL_BIN/archvnde-dock"

echo "============================================="
echo "Reinstall complete!"
echo "Binaries updated at: $LOCAL_BIN"
echo "Default configuration is automatically created at ~/.config/archvnde/style.css on startup."
echo "============================================="
