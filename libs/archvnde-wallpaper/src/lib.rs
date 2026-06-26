use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

/// Checks if a command binary exists in the system's PATH.
fn has_binary(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Sets the desktop wallpaper using the best available backend utility.
/// Supported backends: swww, swaybg, feh.
pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Wallpaper file does not exist at: {:?}", path));
    }

    let path_str = path.to_str().ok_or("Invalid path encoding")?;

    // 1. Try swww (Wayland animated wallpaper daemon)
    if has_binary("swww") {
        let _ = Command::new("swww-daemon").spawn();
        let status = Command::new("swww")
            .args(["img", path_str])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            let _ = save_current_wallpaper_link(path);
            return Ok(());
        }
    }

    // 2. Try swaybg (standard Wayland background setter)
    if has_binary("swaybg") {
        let _ = Command::new("killall").arg("swaybg").output();
        let status = Command::new("swaybg")
            .args(["-i", path_str, "-m", "fill"])
            .spawn();
        if status.is_ok() {
            let _ = save_current_wallpaper_link(path);
            return Ok(());
        }
    }

    // 3. Try feh (X11 backend fallback)
    if has_binary("feh") {
        let status = Command::new("feh")
            .args(["--bg-fill", path_str])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            let _ = save_current_wallpaper_link(path);
            return Ok(());
        }
    }

    Err("No compatible wallpaper backend (swww, swaybg, or feh) was found in PATH".to_string())
}

/// Helper function to save the active wallpaper path in ~/.config/archvnde/
fn save_current_wallpaper_link(path: &Path) -> std::io::Result<()> {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("archvnde");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        let link_file = config_dir.join("current_wallpaper");
        fs::write(link_file, path.to_str().unwrap_or(""))?;
    }
    Ok(())
}

/// Retrieves the path to the currently active wallpaper from user configuration.
pub fn get_current_wallpaper() -> Option<PathBuf> {
    let mut config_dir = dirs::config_dir()?;
    config_dir.push("archvnde/current_wallpaper");
    if config_dir.exists() {
        if let Ok(content) = fs::read_to_string(config_dir) {
            let path = PathBuf::from(content.trim());
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}
