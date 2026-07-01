#!/bin/bash
set -e

echo "============================================="
echo "        ArchVNDE Hot Update & Reload"
echo "============================================="

# 1. Pull latest code
echo "Pulling latest code changes..."
git pull origin main || git pull || true

# 2. Rebuild in release mode
echo "Rebuilding in release mode..."
cargo build --release

# 3. Create local bin and log directories if needed
LOCAL_BIN="$HOME/.local/bin"
LOG_DIR="$HOME/.cache/archvnde"
mkdir -p "$LOCAL_BIN"
mkdir -p "$LOG_DIR"

# 4. Stop running panel/menu/switcher instances
echo "Stopping active processes..."
killall archvnde-panel || true
killall archvnde-menu || true
killall archvnde-switcher || true

# 5. Overwrite binaries in ~/.local/bin
echo "Installing new binaries..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-menu "$LOCAL_BIN/archvnde-menu"
cp target/release/archvnde-switcher "$LOCAL_BIN/archvnde-switcher"
cp target/release/archvnde-screenshot "$LOCAL_BIN/archvnde-screenshot"

# 6. Reload labwc settings
echo "Reloading labwc compositor..."
labwc --reconfigure || true

# 7. Start the panel and redirect stdout/stderr to log file
echo "Starting archvnde-panel..."
killall dunst || true
killall mako || true
killall fnott || true
killall xfce4-notifyd || true
~/.local/bin/archvnde-panel > "$LOG_DIR/panel.log" 2>&1 &

echo "============================================="
echo "Update complete! Streaming panel logs below..."
echo "Press Ctrl+C to exit log streaming."
echo "============================================="
sleep 1
tail -n 30 -f "$LOG_DIR/panel.log"
