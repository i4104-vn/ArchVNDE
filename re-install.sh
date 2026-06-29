#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e
git pull origin

echo "============================================="
echo "   ArchVNDE Fresh Rebuild & Reinstall Script"
echo "============================================="

# 1. Install/Update all dependencies and the Rust toolchain first via pacman
echo "Ensuring Arch Linux packages, development tools, and Rust compiler are installed..."
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
killall archvnde-menu || true
killall archvnde-switcher || true
killall archvnde-notification || true

# Remove old deprecated notification binary
rm -f "$LOCAL_BIN/archvnde-notification"

# 6. Reinstall the binaries
echo "Overwriting binaries in $LOCAL_BIN..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-menu "$LOCAL_BIN/archvnde-menu"
cp target/release/archvnde-switcher "$LOCAL_BIN/archvnde-switcher"

# 7. Write/update labwc configuration files
echo "Updating labwc configuration files..."
mkdir -p "$HOME/.config/labwc"
cat << 'EOF' > "$HOME/.config/labwc/autostart"
#!/bin/bash
# Autostart configuration for labwc with ArchVNDE shell

# Start Wayland wallpaper daemon
awww-daemon &
sleep 0.5

# Set the default wallpaper
awww img wallpaper.png &

# Start ArchVNDE status panel
~/.local/bin/archvnde-panel &
EOF
chmod +x "$HOME/.config/labwc/autostart"

cat << 'EOF' > "$HOME/.config/labwc/rc.xml"
<?xml version="1.0" encoding="UTF-8"?>
<labwc_config>
  <keyboard>
    <default />
    <!-- Override Alt-Tab with custom archvnde-switcher -->
    <keybind key="A-Tab">
      <action name="Execute" command="~/.local/bin/archvnde-switcher" />
    </keybind>
  </keyboard>
  <mouse>
    <default />
    <!-- Custom context menu for desktop right-click -->
    <context name="Root">
      <mousebind button="Right" action="Press">
        <action name="Execute" command="~/.local/bin/archvnde-menu" />
      </mousebind>
    </context>
  </mouse>
</labwc_config>
EOF

# 8. Reload configuration and restart panel
echo "Reloading labwc configuration and starting panel..."
labwc --reconfigure || true
~/.local/bin/archvnde-panel &

echo "============================================="
echo "Reinstall complete!"
echo "Binaries updated at: $LOCAL_BIN"
echo "Default configuration is automatically created at ~/.config/archvnde/style.css on startup."
echo "============================================="
