# ArchVNDE – Application Configs

This directory contains application-specific configuration files designed to match
the ArchVNDE design language: dark glassmorphic surfaces, blue accents (#3b82f6),
Inter font, and subtle translucent borders.

These configs are **independent of the panel/shell source code** and are applied
directly to the user's home directory by the install script.

## Structure

```
configs/
  dolphin/         # KDE Dolphin file manager
    ArchVNDE.colors     # KDE color scheme (Qt/Plasma-compatible)
    dolphinrc           # Dolphin preferences
    kdeglobals          # Global KDE/Qt appearance settings
```

## Applying configs manually

```bash
# Dolphin
cp configs/dolphin/ArchVNDE.colors ~/.local/share/color-schemes/
cp configs/dolphin/dolphinrc       ~/.config/dolphinrc
cp configs/dolphin/kdeglobals      ~/.config/kdeglobals
```

The `install.sh` script runs this automatically.
