use serde::{Deserialize, Serialize};
use std::fs;
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

// Helper to get configuration directory path (~/.config/archvnde)
pub fn get_archvnde_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
        .join("archvnde")
}

const DEFAULT_CSS: &str = r#"/* ArchVNDE Glassmorphism GTK4 Theme */

* {
    font-family: 'Outfit', 'Inter', sans-serif;
}

/* General Window Transparency (Critical for custom shells) */
window, window.background, .panel-window, .quick-settings-window, .launcher-window, .notification-card {
    background: transparent !important;
    background-color: transparent !important;
    box-shadow: none !important;
    border: none !important;
}

/* Panel Status Bar Styles */
.panel-box {
    background-color: rgba(19, 24, 36, 0.15) !important;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
    color: rgba(255, 255, 255, 0.95) !important;
    margin: 0px !important;
    padding: 0px 15px !important;
    min-height: 40px !important;
    border-radius: 0px !important;
}

.panel-title {
    font-weight: bold;
    color: #1f6feb !important;
    font-size: 1.1em;
}

.workspace-box {
    margin-left: 10px;
}

.workspace-button {
    background-color: rgba(39, 45, 62, 0.6) !important;
    background-image: none !important;
    border: 1px solid rgba(255, 255, 255, 0.05) !important;
    border-radius: 8px !important;
    color: rgba(255, 255, 255, 0.6) !important;
    padding: 2px 12px !important;
    font-size: 0.85em;
    font-weight: bold;
    margin-right: 5px;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.workspace-button:hover {
    background-color: rgba(255, 255, 255, 0.1) !important;
    color: #ffffff !important;
}

.workspace-button.active {
    background-color: #1f6feb !important;
    background-image: none !important;
    border-color: #3b82f6 !important;
    color: #ffffff !important;
    box-shadow: 0 0 8px rgba(31, 111, 235, 0.4) !important;
}

.panel-clock {
    font-weight: bold;
    font-size: 0.95em;
    color: rgba(255, 255, 255, 0.95) !important;
}

.panel-settings-btn {
    background-color: rgba(39, 45, 62, 0.6) !important;
    background-image: none !important;
    border: 1px solid rgba(255, 255, 255, 0.05) !important;
    border-radius: 8px !important;
    color: rgba(255, 255, 255, 0.95) !important;
    padding: 2px 14px !important;
    font-weight: bold;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.panel-settings-btn:hover {
    background-color: rgba(255, 255, 255, 0.1) !important;
}

/* Quick Settings Styles */
.quick-settings-box {
    background-color: rgba(19, 24, 36, 0.75) !important;
    border: 1px solid rgba(255, 255, 255, 0.08) !important;
    border-radius: 16px !important;
    color: rgba(255, 255, 255, 0.95) !important;
    padding: 20px !important;
    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3) !important;
}

.quick-settings-title {
    font-weight: bold;
    color: #1f6feb !important;
    font-size: 1.2em;
    margin-bottom: 10px;
}

.quick-tile {
    background-color: rgba(39, 45, 62, 0.6) !important;
    background-image: none !important;
    border: 1px solid rgba(255, 255, 255, 0.05) !important;
    border-radius: 12px !important;
    color: rgba(255, 255, 255, 0.95) !important;
    padding: 10px !important;
    font-weight: bold;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.quick-tile:hover {
    background-color: rgba(255, 255, 255, 0.1) !important;
}

.quick-tile.active {
    background-color: rgba(255, 255, 255, 0.95) !important;
    background-image: none !important;
    color: #121620 !important;
    border-color: #ffffff !important;
    box-shadow: 0 4px 12px rgba(255, 255, 255, 0.2) !important;
}

/* Capsule Sliders for Volume & Brightness */
scale trough {
    min-height: 12px;
    border-radius: 6px;
    background-color: rgba(255, 255, 255, 0.1) !important;
    border: none;
}

scale progress {
    background-image: linear-gradient(to right, #1f6feb, #3b82f6) !important;
}

scale slider {
    min-width: 14px;
    min-height: 14px;
    margin: -2px;
    border-radius: 50% !important;
    background-color: #ffffff !important;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4) !important;
    border: none;
    transition: transform 0.1s ease;
}

scale slider:hover {
    transform: scale(1.2);
}

/* Launcher Styles */
.launcher-box {
    background-color: rgba(12, 16, 25, 0.9) !important;
    border: 1px solid rgba(255, 255, 255, 0.09) !important;
    border-radius: 24px !important;
    color: rgba(255, 255, 255, 0.95) !important;
    padding: 24px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4) !important;
}

.launcher-search entry {
    background-color: rgba(255, 255, 255, 0.06) !important;
    border: 1px solid rgba(255, 255, 255, 0.08) !important;
    border-radius: 14px !important;
    color: #ffffff !important;
    padding: 12px 18px !important;
    font-size: 1.1em;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.launcher-search entry:focus {
    border-color: #3b82f6 !important;
    box-shadow: 0 0 12px rgba(59, 130, 246, 0.35) !important;
    background-color: rgba(255, 255, 255, 0.08) !important;
}

.launcher-list {
    background: transparent;
}

.launcher-list row {
    background-color: transparent !important;
    padding: 12px 16px !important;
    border-radius: 12px !important;
    margin: 5px 0 !important;
    color: rgba(255, 255, 255, 0.85) !important;
    transition: all 0.15s ease-in-out;
}

.launcher-list row:hover {
    background-color: rgba(255, 255, 255, 0.06) !important;
    color: #ffffff !important;
}

.launcher-list row:selected {
    background-image: linear-gradient(135deg, #1f6feb 0%, #3b82f6 100%) !important;
    color: #ffffff !important;
}

/* Notification Daemon Styles */
.notification-box {
    background-color: rgba(12, 16, 25, 0.92) !important;
    border: 1px solid rgba(255, 255, 255, 0.1) !important;
    border-radius: 18px !important;
    color: rgba(255, 255, 255, 0.95) !important;
    padding: 16px 20px !important;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.4) !important;
}

.notification-title {
    font-weight: 700;
    color: #3b82f6;
    font-size: 1.15em;
    letter-spacing: 0.3px;
}

.notification-body {
    color: rgba(255, 255, 255, 0.75) !important;
    font-size: 0.95em;
}
"#;

/// Initializes the user configuration directory, writes the default
/// Glassmorphism stylesheet if missing, and registers it with GTK.
pub fn init_theme() {
    let config_dir = get_archvnde_config_dir();
    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("Failed to create config directory: {}", e);
            return;
        }
    }

    let css_path = config_dir.join("style.css");
    // Always write the latest default CSS stylesheet to ensure style updates are applied immediately
    if let Err(e) = fs::write(&css_path, DEFAULT_CSS) {
        eprintln!("Failed to write default stylesheet: {}", e);
        return;
    }
    println!("Updated glassmorphism stylesheet at {:?}", css_path);

    // Load CSS into GTK
    let provider = gtk4::CssProvider::new();
    
    // Connect to parsing-error to catch and print any CSS syntax errors to stderr
    provider.connect_parsing_error(|_provider, section, error| {
        eprintln!(
            "GTK CSS Parsing Error: {} in section {:?}",
            error.message(),
            section
        );
    });

    provider.load_from_path(css_path);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );
        println!("Successfully registered glassmorphism stylesheet with GTK Display.");
    } else {
        eprintln!("Failed to get default GDK display, theme styling might not apply.");
    }
}
