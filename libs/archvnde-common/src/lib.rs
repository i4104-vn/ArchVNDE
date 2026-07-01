pub mod config;
pub mod theme;
pub mod animation;
pub mod icon;
pub mod island;
pub mod models;
pub mod desktop;
pub mod window;

pub use models::{ThemeConfig, ShellConfig, IslandState};
pub use config::get_archvnde_config_dir;
pub use theme::init_theme;
pub use island::{update_island_state, clear_island_state, get_island_state_path};
pub use desktop::{find_desktop_apps, DesktopApp};
