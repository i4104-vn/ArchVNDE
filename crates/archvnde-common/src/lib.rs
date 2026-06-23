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
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tdkhoa-01".to_string());
    PathBuf::from(home).join(".config").join("archvnde")
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

/* General Window Properties */
window {
    background: transparent;
}

/* Panel Status Bar Styles */
.panel-window {
    background-color: @bg-glass-dark;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    color: @text-primary;
}

.panel-title {
    font-weight: bold;
    color: @color-accent;
}

/* Launcher Styles */
.launcher-window {
    background-color: @bg-glass-dark;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 16px;
    color: @text-primary;
}

.launcher-search entry {
    background-color: @bg-card-inactive;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    color: @text-primary;
    padding: 8px 12px;
}

.launcher-list {
    background: transparent;
}

.launcher-list row {
    background-color: transparent;
    padding: 10px;
    border-radius: 8px;
    margin: 2px 0;
    color: @text-primary;
    transition: all 0.2s ease-in-out;
}

.launcher-list row:hover {
    background-color: @bg-card-inactive;
}

.launcher-list row:selected {
    background-color: @color-accent;
    color: @text-primary;
}

/* Notification Daemon Styles */
.notification-card {
    background-color: @bg-glass-dark;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    color: @text-primary;
}

.notification-title {
    font-weight: bold;
    color: @color-accent;
    font-size: 1.1em;
}

.notification-body {
    color: @text-secondary;
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
    if !css_path.exists() {
        if let Err(e) = fs::write(&css_path, DEFAULT_CSS) {
            eprintln!("Failed to write default stylesheet: {}", e);
            return;
        }
        println!("Created default glassmorphism stylesheet at {:?}", css_path);
    }

    // Load CSS into GTK
    let provider = gtk4::CssProvider::new();
    provider.load_from_path(css_path);

    if let Some(display) = gdk4::Display::default() {
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
