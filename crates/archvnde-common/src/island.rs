use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

use crate::models::IslandState;


pub fn get_island_state_path() -> PathBuf {
    std::env::temp_dir().join("archvnde-island.toml")
}

pub fn update_island_state(state: &IslandState) -> Result<(), std::io::Error> {
    let path = get_island_state_path();
    if let Ok(toml_str) = toml::to_string(state) {
        fs::write(path, toml_str)?;
    }
    Ok(())
}

pub fn clear_island_state() -> Result<(), std::io::Error> {
    let path = get_island_state_path();
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    Ok(())
}
