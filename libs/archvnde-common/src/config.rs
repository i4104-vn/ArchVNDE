use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::models::{ThemeConfig, ShellConfig};


// Helper to get configuration directory path (~/.config/archvnde)
pub fn get_archvnde_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
        .join("archvnde")
}
