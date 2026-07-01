use gtk4::prelude::*;
use crate::widgets;

pub fn build_lock_ui(app: &gtk4::Application, wallpaper_path: &str) {
    // Inject dynamic wallpaper background CSS globally for all lock windows
    let provider = gtk4::CssProvider::new();
    let custom_css = format!(
        ".lock-window {{ background-image: url('file://{}'); background-size: cover; background-position: center; }}",
        wallpaper_path
    );
    provider.load_from_data(&custom_css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Get connected monitors
    let display = gtk4::gdk::Display::default().expect("Failed to get default GDK display");
    let monitors = display.monitors();
    let num_monitors = monitors.n_items();

    if num_monitors == 0 {
        // Fallback for systems returning no monitor info
        widgets::create_lock_window(app, None, true);
    } else {
        // Spawn a lock window on every monitor to ensure all screens are completely covered
        for i in 0..num_monitors {
            if let Some(monitor) = monitors.item(i).and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok()) {
                let is_primary = i == 0;
                widgets::create_lock_window(app, Some(&monitor), is_primary);
            }
        }
    }
}
