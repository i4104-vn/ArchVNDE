pub mod core;
pub mod theme;
pub mod animation;
pub mod icon;
pub mod island;
pub mod models;
pub mod window;

pub use models::{ThemeConfig, ShellConfig, IslandState};
pub use core::config::get_archvnde_config_dir;
pub use theme::init_theme;
pub use theme::apply_theme_class;
pub use island::{update_island_state, clear_island_state, get_island_state_path};
pub use core::desktop::{find_desktop_apps, DesktopApp};
pub use core::desktop;
pub use window::init_layer_window;
pub use archvnde_i18n as i18n;
