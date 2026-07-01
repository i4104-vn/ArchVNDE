#[path = "src/config.rs"]
pub mod config;
#[path = "src/theme.rs"]
pub mod theme;
#[path = "src/animation/mod.rs"]
pub mod animation;
#[path = "src/icon/mod.rs"]
pub mod icon;
#[path = "src/island.rs"]
pub mod island;
#[path = "src/models/mod.rs"]
pub mod models;


pub use models::{ThemeConfig, ShellConfig, IslandState};
pub use config::get_archvnde_config_dir;
pub use theme::init_theme;
pub use island::{update_island_state, clear_island_state, get_island_state_path};
