#!/bin/bash

# Ensure local bin is in PATH
export PATH="$HOME/.local/bin:$PATH"
export GSK_RENDERER=cairo

# Write config files for labwc
mkdir -p "$HOME/.config/labwc"
AUTOSTART_FILE="$HOME/.config/labwc/autostart"
RC_FILE="$HOME/.config/labwc/rc.xml"

echo "Stopping any running shell processes..."
killall archvnde-panel || true
killall archvnde-menu || true
killall dunst || true
killall mako || true
killall fnott || true
killall xfce4-notifyd || true 

# Setup default autostart
cat << 'EOF' > "$AUTOSTART_FILE"
#!/bin/bash
# Autostart configuration for labwc with ArchVNDE shell

# Start Wayland wallpaper daemon
awww-daemon &
sleep 0.5

# Set the default wallpaper
awww img wallpaper.png &

# Start ArchVNDE status panel
mkdir -p "$HOME/.cache/archvnde"
archvnde-panel > "$HOME/.cache/archvnde/panel.log" 2>&1 &
EOF
chmod +x "$AUTOSTART_FILE"
echo "Configured labwc autostart at $AUTOSTART_FILE"

# Setup default rc.xml with right-click menu override
cat << 'EOF' > "$RC_FILE"
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
echo "Configured labwc rc.xml at $RC_FILE"

# Run labwc with software rendering variables for virtual machine compatibility
export WLR_RENDERER=pixman
export WLR_NO_HARDWARE_CURSORS=1

echo "============================================="
echo "Starting labwc compositor with ArchVNDE..."
echo "Press Ctrl+Alt+Backspace to exit labwc."
echo "============================================="

exec labwc
