#!/bin/bash
set -e

echo "============================================="
echo "   ArchVNDE Desktop Shell Installation Script"
echo "============================================="

# 1. Install all dependencies and the Rust toolchain via pacman
echo "Installing Arch Linux packages, development tools, and Rust compiler..."
sudo pacman -S --needed --noconfirm base-devel git pkgconf gtk4 gtk4-layer-shell rust labwc meson ninja playerctl papirus-icon-theme grim ttf-inter ttf-ubuntu-font-family ttf-jetbrains-mono-nerd otf-font-awesome

# 2. Install wlrctl from AUR if not present (required by the window switcher)
if ! command -v wlrctl &> /dev/null; then
    echo "wlrctl not found, installing from AUR..."
    rm -rf /tmp/wlrctl
    git clone https://aur.archlinux.org/wlrctl.git /tmp/wlrctl
    cd /tmp/wlrctl
    makepkg -si --noconfirm
    cd -
fi

# 3. Check and build wtype from source (required to fix Alt modifier release states)
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"
if [ ! -f "$LOCAL_BIN/wtype" ]; then
    echo "wtype not found, compiling from source..."
    rm -rf /tmp/wtype
    git clone https://github.com/atx/wtype.git /tmp/wtype
    cd /tmp/wtype
    meson setup build
    ninja -C build
    cp build/wtype "$LOCAL_BIN/wtype"
    cd -
fi

# 4. Clean and rebuild the workspace in release mode
echo "Cleaning and compiling ArchVNDE components in release mode..."
cargo clean
cargo build --release

# 5. Stop running panel/menu/switcher instances
echo "Stopping active processes..."
killall archvnde-panel || true
killall archvnde-menu || true
killall archvnde-switcher || true
killall archvnde-screenshot || true

# 6. Install the binaries
echo "Installing binaries to $LOCAL_BIN..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-menu "$LOCAL_BIN/archvnde-menu"
cp target/release/archvnde-switcher "$LOCAL_BIN/archvnde-switcher"
cp target/release/archvnde-screenshot "$LOCAL_BIN/archvnde-screenshot"

# 7. Write/update labwc configuration files
echo "Configuring labwc compositor integrations..."
mkdir -p "$HOME/.config/labwc"

# Create autostart script
cat << 'EOF' > "$HOME/.config/labwc/autostart"
#!/bin/bash
# Autostart configuration for labwc with ArchVNDE shell

# Start Wayland wallpaper daemon
awww-daemon &
sleep 0.5

# Set the default wallpaper
awww img wallpaper.png &

# Start ArchVNDE status panel
mkdir -p "$HOME/.cache/archvnde"
~/.local/bin/archvnde-panel > "$HOME/.cache/archvnde/panel.log" 2>&1 &
EOF
chmod +x "$HOME/.config/labwc/autostart"

# Create rc.xml configuration
cat << 'EOF' > "$HOME/.config/labwc/rc.xml"
<?xml version="1.0" encoding="UTF-8"?>
<labwc_config>
  <keyboard>
    <default />
    <!-- Override Alt-Tab with custom archvnde-switcher -->
    <keybind key="A-Tab">
      <action name="Execute" command="~/.local/bin/archvnde-switcher" />
    </keybind>
    <!-- Take screenshot with Win+Shift+S -->
    <keybind key="W-S-s">
      <action name="Execute" command="~/.local/bin/archvnde-screenshot" />
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
mkdir -p "$HOME/.cache/archvnde"
~/.local/bin/archvnde-panel > "$HOME/.cache/archvnde/panel.log" 2>&1 &

echo "============================================="
echo "Installation & Setup complete!"
echo "Binaries installed to: $LOCAL_BIN"
echo "Log file location: ~/.cache/archvnde/panel.log"
echo "You can launch labwc to use the full desktop shell."
echo "============================================="
