use std::fs;
use crate::config::get_archvnde_config_dir;

const DEFAULT_CSS: &str = concat!(
    include_str!("styles/bar.css"),
    "\n",
    include_str!("styles/button.css"),
    "\n",
    include_str!("styles/panel.css"),
    "\n",
    include_str!("styles/power.css")
);

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
