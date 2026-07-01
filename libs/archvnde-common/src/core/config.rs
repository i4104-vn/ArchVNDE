//! Configuration path resolvers.

use std::path::PathBuf;

/// Resolves the absolute directory path to the user's config folder: `~/.config/archvnde/`.
pub fn get_archvnde_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
        .join("archvnde")
}

