#!/bin/bash
set -e

echo "============================================="
echo "   ArchVNDE Desktop Shell Installation Script"
echo "============================================="

# 1. Install all dependencies, the Rust toolchain, and system fonts via pacman
echo "Installing Arch Linux packages..."
sudo pacman -Syu --needed --noconfirm base-devel git pkgconf gtk4 gtk4-layer-shell rust labwc meson ninja playerctl grim wl-clipboard

# Check if yay is installed, and install it from AUR if missing
if ! command -v yay &> /dev/null; then
    echo "yay not found, installing yay-bin from AUR..."
    rm -rf /tmp/yay-bin
    git clone https://aur.archlinux.org/yay-bin.git /tmp/yay-bin
    cd /tmp/yay-bin
    makepkg -si --noconfirm
    cd -
fi

# Install AUR packages using yay
yay -S --noconfirm dolphin github-desktop fastfetch neovim awww brightnessctl 
yay -S --noconfirm inter-font ttf-ubuntu-font-family ttf-jetbrains-mono-nerd otf-font-awesome ttf-nerd-fonts-symbols
yay -S --noconfirm papirus-icon-theme kvantum-qt5

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

# 5. Stop running panel/menu/switcher/lock instances
echo "Stopping active processes..."
killall archvnde-panel || true
killall archvnde-menu || true
killall archvnde-switcher || true
killall archvnde-screenshot || true
killall archvnde-lock || true

# 6. Install the binaries
echo "Installing binaries to $LOCAL_BIN..."
cp target/release/archvnde-panel "$LOCAL_BIN/archvnde-panel"
cp target/release/archvnde-menu "$LOCAL_BIN/archvnde-menu"
cp target/release/archvnde-switcher "$LOCAL_BIN/archvnde-switcher"
cp target/release/archvnde-screenshot "$LOCAL_BIN/archvnde-screenshot"
cp target/release/archvnde-lock "$LOCAL_BIN/archvnde-lock"

# Copy wallpaper to standard config dir
mkdir -p "$HOME/.config/archvnde"
cp wallpaper.png "$HOME/.config/archvnde/wallpaper.png"

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
awww img "$HOME/.config/archvnde/wallpaper.png" &

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
    <!-- Lock screen with Win+L -->
    <keybind key="W-l">
      <action name="Execute" command="~/.local/bin/archvnde-lock" />
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

# 9. Configure system-wide default fonts (Inter & JetBrains Mono)
echo "Configuring system-wide default fonts for GTK and Fontconfig..."
mkdir -p "$HOME/.config/gtk-3.0" "$HOME/.config/gtk-4.0" "$HOME/.config/fontconfig"

# GTK 3.0 settings
cat << 'EOF' > "$HOME/.config/gtk-3.0/settings.ini"
[Settings]
gtk-font-name=Inter 11
gtk-icon-theme-name=Adwaita
EOF

# GTK 4.0 settings
cat << 'EOF' > "$HOME/.config/gtk-4.0/settings.ini"
[Settings]
gtk-font-name=Inter 11
gtk-icon-theme-name=Adwaita
EOF

# Fontconfig default aliases
cat << 'EOF' > "$HOME/.config/fontconfig/fonts.conf"
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <!-- Default sans-serif font -->
  <match target="pattern">
    <test qual="any" name="family"><string>sans-serif</string></test>
    <edit name="family" mode="assign" binding="same">
      <string>Inter</string>
    </edit>
  </match>

  <!-- Default serif font -->
  <match target="pattern">
    <test qual="any" name="family"><string>serif</string></test>
    <edit name="family" mode="assign" binding="same">
      <string>Inter</string>
    </edit>
  </match>

  <!-- Default monospace font -->
  <match target="pattern">
    <test qual="any" name="family"><string>monospace</string></test>
    <edit name="family" mode="assign" binding="same">
      <string>JetBrainsMono Nerd Font</string>
    </edit>
  </match>
</fontconfig>
EOF

# Rebuild font cache
echo "Rebuilding font cache..."
fc-cache -fv || true

# 10. Apply ArchVNDE app configs (Dolphin, KDE globals, color scheme)
echo "Applying ArchVNDE application configurations..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# KDE color scheme
mkdir -p "$HOME/.local/share/color-schemes"
cp "$SCRIPT_DIR/configs/dolphin/ArchVNDE.colors" "$HOME/.local/share/color-schemes/ArchVNDE.colors"

# Dolphin preferences
mkdir -p "$HOME/.config"
cp "$SCRIPT_DIR/configs/dolphin/dolphinrc" "$HOME/.config/dolphinrc"

# KDE global appearance (colors, fonts, icons for all Qt apps)
cp "$SCRIPT_DIR/configs/dolphin/kdeglobals" "$HOME/.config/kdeglobals"

echo "App configurations applied."

echo "============================================="
echo "Installation & Setup complete!"
echo "Binaries installed to: $LOCAL_BIN"
echo "Log file location: ~/.cache/archvnde/panel.log"
echo "You can launch labwc to use the full desktop shell."
echo "============================================="
