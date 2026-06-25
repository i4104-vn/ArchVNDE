use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinnedApp {
    pub name: String,
    pub icon: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockConfig {
    pub pinned_apps: Vec<PinnedApp>,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            pinned_apps: vec![
                PinnedApp {
                    name: "Terminal".to_string(),
                    icon: "terminal".to_string(),
                    command: "foot".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "Files".to_string(),
                    icon: "folder".to_string(),
                    command: "pcmanfm".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "Web Browser".to_string(),
                    icon: "search".to_string(),
                    command: "firefox".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "Music Player".to_string(),
                    icon: "music".to_string(),
                    command: "amberol".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "System Settings".to_string(),
                    icon: "settings".to_string(),
                    command: "gnome-control-center".to_string(),
                    args: vec![],
                },
            ],
        }
    }
}

pub fn get_dock_config_path() -> PathBuf {
    archvnde_common::get_archvnde_config_dir().join("dock.toml")
}

pub fn load_dock_config() -> DockConfig {
    let path = get_dock_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<DockConfig>(&content) {
                return config;
            }
        }
    }
    
    // Save default config if not exists
    let default_config = DockConfig::default();
    let _ = save_dock_config(&default_config);
    default_config
}

pub fn save_dock_config(config: &DockConfig) -> Result<(), std::io::Error> {
    let path = get_dock_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string(config).unwrap_or_default();
    fs::write(path, toml_str)?;
    Ok(())
}
