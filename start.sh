#!/bin/bash

# Ensure local bin is in PATH
export PATH="$HOME/.local/bin:$PATH"

# Write autostart for labwc if it does not exist
mkdir -p "$HOME/.config/labwc"
AUTOSTART_FILE="$HOME/.config/labwc/autostart"

# Setup default autostart if missing
if [ ! -f "$AUTOSTART_FILE" ]; then
    cat << 'EOF' > "$AUTOSTART_FILE"
#!/bin/bash
# Autostart configuration for labwc with ArchVNDE shell

# Start ArchVNDE status panel
archvnde-panel &

# Start ArchVNDE notification daemon
archvnde-notification &
EOF
    chmod +x "$AUTOSTART_FILE"
    echo "Configured labwc autostart at $AUTOSTART_FILE"
fi

# Run labwc with software rendering variables for virtual machine compatibility
export WLR_RENDERER=pixman
export WLR_NO_HARDWARE_CURSORS=1

echo "============================================="
echo "Starting labwc compositor with ArchVNDE..."
echo "Press Ctrl+Alt+Backspace to exit labwc."
echo "============================================="

exec labwc
