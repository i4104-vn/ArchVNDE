// Trigger rebuild to update embedded CSS resources
use std::fs;
use crate::core::config::get_archvnde_config_dir;

const DEFAULT_CSS: &str = concat!(
    include_str!("styles/bar.css"),
    "\n",
    include_str!("styles/button.css"),
    "\n",
    include_str!("styles/control_center.css"),
    "\n",
    include_str!("styles/launcher.css"),
    "\n",
    include_str!("styles/notification.css"),
    "\n",
    include_str!("styles/calendar.css"),
    "\n",
    include_str!("styles/power.css"),
    "\n",
    include_str!("styles/menu.css"),
    "\n",
    include_str!("styles/switcher.css"),
    "\n",
    include_str!("styles/screenshot.css")
);

/// Initializes the user configuration directory, writes the default
/// Glassmorphism stylesheet if missing, and registers it with GTK.
pub fn init_theme() {
    // Clean Windows CRLF line endings to LF before compiling/parsing to prevent GTK4 CSS parser errors
    let cleaned_css = DEFAULT_CSS.replace("\r\n", "\n");

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

    provider.load_from_data(&cleaned_css);

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
