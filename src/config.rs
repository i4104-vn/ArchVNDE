use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub blur_tint: String,
    pub blur_opacity: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    pub corner_radius: u32,
    pub border_width: u32,
    pub border_color: String,
    pub blur_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutConfig {
    pub launch_terminal: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
