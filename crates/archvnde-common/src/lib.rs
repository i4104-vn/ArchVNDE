use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub blur_radius: u32,
    pub opacity: f64,
    pub border_color: String,
    pub border_width: u32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            blur_radius: 20,
            opacity: 0.75,
            border_color: "#ffffff".to_string(),
            border_width: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellConfig {
    pub theme: ThemeConfig,
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/home/tdkhoa-01/.config"))
        .join("archvnde")
}

// Custom dirs crate fallback helper since we don't have dirs crate in deps yet, 
// let's just make it simple using std::env.
pub fn get_archvnde_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tdkhoa-01".to_string());
    PathBuf::from(home).join(".config").join("archvnde")
}
