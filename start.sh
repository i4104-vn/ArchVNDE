#!/bin/bash

# Ensure local bin is in PATH
export PATH="$HOME/.local/bin:$PATH"

# Write config files for labwc
mkdir -p "$HOME/.config/labwc"
AUTOSTART_FILE="$HOME/.config/labwc/autostart"
RC_FILE="$HOME/.config/labwc/rc.xml"

# Setup default autostart
cat << 'EOF' > "$AUTOSTART_FILE"
#!/bin/bash
# Autostart configuration for labwc with ArchVNDE shell

# Start Wayland wallpaper daemon
swww-daemon &
sleep 0.5

# Set the default wallpaper
swww img /home/tdkhoa-01/Documents/src/I4104/ArchVNDE/wallpaper.png &

# Start ArchVNDE status panel
archvnde-panel &

# Start ArchVNDE notification daemon
archvnde-notification &
EOF
chmod +x "$AUTOSTART_FILE"
echo "Configured labwc autostart at $AUTOSTART_FILE"

# Setup default rc.xml with right-click menu override
cat << 'EOF' > "$RC_FILE"
<?xml version="1.0" encoding="UTF-8"?>
<labwc_config>
  <keyboard>
    <default />
  </keyboard>
  <mouse>
    <default />
    <!-- Custom context menu for desktop right-click -->
    <context name="Root">
      <mousebind button="Right" action="Press">
        <action name="Execute" command="archvnde-menu" />
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
