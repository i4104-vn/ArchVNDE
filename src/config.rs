use serde::{Deserialize, Serialize};

/// Visual theme settings (blur tint colour and opacity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Background tint colour for glass windows, as a `#RRGGBB` hex string.
    pub blur_tint: String,
    /// Alpha channel applied on top of the blurred background (`0.0`–`1.0`).
    pub blur_opacity: f32,
}

/// Window decoration defaults applied to all client surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Corner rounding radius in logical pixels.
    pub corner_radius: u32,
    /// Border thickness in logical pixels.
    pub border_width: u32,
    /// Border colour as a `#RRGGBB` hex string.
    pub border_color: String,
    /// Whether glassmorphism blur is enabled by default for new windows.
    pub blur_enabled: bool,
}

/// Keyboard shortcut configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Executable launched by the Super+Return binding.
    pub launch_terminal: String,
}

/// Root compositor configuration, loaded from `~/.config/glass-wm/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub window: WindowConfig,
    pub shortcut: ShortcutConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                blur_tint: "#14191f".to_string(),
                blur_opacity: 0.45,
            },
            window: WindowConfig {
                corner_radius: 12,
                border_width: 1,
                border_color: "#ffffff".to_string(),
                blur_enabled: true,
            },
            shortcut: ShortcutConfig {
                launch_terminal: "alacritty".to_string(),
            },
        }
    }
}
