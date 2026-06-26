use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub use crate::models::{PinnedApp, DockConfig};


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
