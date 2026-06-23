pub mod config;
pub mod theme;

pub use config::{ThemeConfig, ShellConfig, get_archvnde_config_dir};
pub use theme::init_theme;
