pub mod config;
pub mod theme;
pub mod animation;
pub mod icon;

pub use config::{ThemeConfig, ShellConfig, get_archvnde_config_dir};
pub use theme::init_theme;

