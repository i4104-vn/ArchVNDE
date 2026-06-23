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

@define-color bg-glass-dark rgba(19, 24, 36, 0.75);
@define-color bg-glass-sidebar rgba(12, 16, 25, 0.45);
@define-color bg-card-active rgba(255, 255, 255, 0.95);
@define-color bg-card-inactive rgba(39, 45, 62, 0.6);

@define-color color-accent #1f6feb;
@define-color color-accent-hover #3b82f6;

@define-color btn-close #ff5f56;
@define-color btn-minimize #ffbd2e;
@define-color btn-maximize #27c93f;

@define-color text-primary rgba(255, 255, 255, 0.95);
@define-color text-secondary rgba(160, 174, 192, 0.9);
@define-color text-active-dark #121620;

/* General Window Transparency (Critical for custom shells) */
window, window decoration, window .background {
    background: transparent !important;
    background-color: transparent !important;
    box-shadow: none !important;
    border: none !important;
}

/* Panel Status Bar Styles */
.panel-box {
    background-color: @bg-glass-dark;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    color: @text-primary;
    padding: 2px 15px;
}

.panel-title {
    font-weight: bold;
    color: @color-accent;
    font-size: 1.1em;
}

.workspace-box {
    margin-left: 15px;
}

.workspace-button {
    background-color: rgba(255, 255, 255, 0.05) !important;
    background-image: none !important;
    border: 1px solid rgba(255, 255, 255, 0.08) !important;
    border-radius: 8px !important;
    color: @text-primary !important;
    padding: 4px 12px;
    font-size: 0.85em;
    font-weight: bold;
    margin-right: 5px;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.workspace-button:hover {
    background-color: rgba(255, 255, 255, 0.12) !important;
    border-color: rgba(255, 255, 255, 0.15) !important;
}

.workspace-button.active {
    background-color: @color-accent !important;
    border-color: @color-accent-hover !important;
    color: @text-primary !important;
    box-shadow: 0 2px 8px rgba(31, 111, 235, 0.3) !important;
}

.panel-clock {
    font-weight: bold;
    font-size: 0.95em;
    color: @text-primary;
}

.panel-settings-btn {
    background-color: rgba(255, 255, 255, 0.05) !important;
    background-image: none !important;
    border: 1px solid rgba(255, 255, 255, 0.08) !important;
    border-radius: 8px !important;
    color: @text-primary !important;
    padding: 4px 12px;
    font-weight: bold;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.panel-settings-btn:hover {
    background-color: rgba(255, 255, 255, 0.12) !important;
    border-color: rgba(255, 255, 255, 0.15) !important;
}

/* Quick Settings Styles */
.quick-settings-box {
    background-color: @bg-glass-dark;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    color: @text-primary;
    padding: 20px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.quick-settings-title {
    font-weight: bold;
    color: @color-accent;
    font-size: 1.3em;
    margin-bottom: 10px;
}

.quick-tile {
    background-color: rgba(255, 255, 255, 0.05) !important;
    background-image: none !important;
    border: 1px solid rgba(255, 255, 255, 0.08) !important;
    border-radius: 12px !important;
    color: @text-primary !important;
    padding: 12px;
    font-weight: bold;
    box-shadow: none !important;
    transition: all 0.2s ease-in-out;
}

.quick-tile:hover {
    background-color: rgba(255, 255, 255, 0.12) !important;
    border-color: rgba(255, 255, 255, 0.15) !important;
}

.quick-tile.active {
    background-color: @bg-card-active !important;
    color: @text-active-dark !important;
    border-color: #ffffff !important;
    box-shadow: 0 4px 12px rgba(255, 255, 255, 0.2) !important;
}

/* Capsule Sliders for Volume & Brightness */
scale trough {
    min-height: 12px;
    border-radius: 6px;
    background-color: rgba(255, 255, 255, 0.1);
    border: none;
}

scale progress {
    min-height: 12px;
    border-radius: 6px;
    background-color: @color-accent;
}

scale slider {
    min-width: 14px;
    min-height: 14px;
    margin: -1px;
    border-radius: 50%;
    background-color: #ffffff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    border: none;
}

/* Launcher Styles */
.launcher-box {
    background-color: @bg-glass-dark;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    color: @text-primary;
    padding: 20px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.launcher-search entry {
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    color: #ffffff;
    padding: 10px 16px;
    font-size: 1.1em;
}

.launcher-search entry:focus {
    border-color: @color-accent;
    box-shadow: 0 0 0 2px rgba(31, 111, 235, 0.2);
}

.launcher-list {
    background: transparent;
}

.launcher-list row {
    background-color: transparent;
    padding: 12px;
    border-radius: 10px;
    margin: 4px 0;
    color: @text-primary;
    transition: all 0.15s ease;
}

.launcher-list row:hover {
    background-color: rgba(255, 255, 255, 0.08);
}

.launcher-list row:selected {
    background-color: @color-accent;
    color: #ffffff;
}

/* Notification Daemon Styles */
.notification-box {
    background-color: rgba(19, 24, 36, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 14px;
    color: @text-primary;
    padding: 12px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.notification-title {
    font-weight: bold;
    color: @color-accent;
    font-size: 1.15em;
}

.notification-body {
    color: @text-secondary;
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
    provider.load_from_path(css_path);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        println!("Successfully registered glassmorphism stylesheet with GTK Display.");
    } else {
        eprintln!("Failed to get default GDK display, theme styling might not apply.");
    }
}
