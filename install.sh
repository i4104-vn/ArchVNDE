#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "============================================="
echo "   ArchVNDE Desktop Shell Installation Script"
echo "============================================="

# 1. Install all dependencies and the Rust toolchain first via pacman
echo "Installing Arch Linux packages, development tools, and Rust compiler..."
sudo pacman -S --needed --noconfirm base-devel git pkgconf gtk4 gtk4-layer-shell rust labwc meson scdoc ninja playerctl papirus-icon-theme 

# Install wlrctl from AUR if not present
if ! command -v wlrctl &> /dev/null; then
    echo "wlrctl not found, installing from AUR..."
    rm -rf /tmp/wlrctl
    git clone https://aur.archlinux.org/wlrctl.git /tmp/wlrctl
    cd /tmp/wlrctl
    makepkg -si --noconfirm
    cd -
fi

# 2. Compile the workspace in release mode
echo "Compiling ArchVNDE components in release mode..."
cargo build --release

# 3. Create local bin directory if it doesn't exist
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"

# 4. Kill any running instances first so the new binaries can be loaded
echo "Stopping any running shell processes..."
killall archvnde-panel || true
killall archvnde-launcher || true
killall archvnde-menu || true
killall archvnde-switcher || true

# 5. Install the binaries
echo "Installing binaries to $LOCAL_BIN..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-launcher "$LOCAL_BIN/archvnde-launcher"
cp target/release/archvnde-menu "$LOCAL_BIN/archvnde-menu"
cp target/release/archvnde-switcher "$LOCAL_BIN/archvnde-switcher"

echo "============================================="
echo "Installation complete!"
echo "Binaries installed to: $LOCAL_BIN"
echo "Make sure '$LOCAL_BIN' is in your PATH."
echo "You can now run 'archvnde-panel', 'archvnde-launcher', or 'archvnde-menu'."
echo "Default configuration is automatically created at ~/.config/archvnde/style.css on startup."
echo "============================================="
