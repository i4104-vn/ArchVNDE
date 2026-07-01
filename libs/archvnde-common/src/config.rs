//! Configuration path resolvers.

use std::path::PathBuf;

<<<<<<< HEAD:libs/archvnde-common/src/config.rs

// Helper to get configuration directory path (~/.config/archvnde)
=======
/// Resolves the absolute directory path to the user's config folder: `~/.config/archvnde/`.
>>>>>>> 52145a1 (refactor: clean up comments and add i18n support):libs/archvnde-common/src/core/config.rs
pub fn get_archvnde_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
        .join("archvnde")
}

