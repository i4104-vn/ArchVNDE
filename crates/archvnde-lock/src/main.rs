mod pam;
mod widgets;

use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};

fn main() {
    println!("Starting ArchVNDE Screen Locker...");

    // 1. Simple argument parsing for custom wallpaper image
    let args: Vec<String> = std::env::args().collect();
    let mut custom_image = None;
    if args.len() > 1 {
        let mut i = 1;
        while i < args.len() {
            if (args[i] == "--image" || args[i] == "-i") && i + 1 < args.len() {
                custom_image = Some(args[i + 1].clone());
                i += 2;
            } else if !args[i].starts_with('-') {
                custom_image = Some(args[i].clone());
                i += 1;
            } else {
                i += 1;
            }
        }
    }

    // Resolve wallpaper path (fallback to standard ~/.config/archvnde/wallpaper.png)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/i4104".to_string());
    let wallpaper_path = if let Some(ref path) = custom_image {
        if std::path::Path::new(path).exists() {
            path.clone()
        } else {
            println!("Custom image at {} not found, falling back to default.", path);
            format!("{}/.config/archvnde/wallpaper.png", home)
        }
    } else {
        format!("{}/.config/archvnde/wallpaper.png", home)
    };

    let application = gtk4::Application::new(
        Some("org.archvnde.lock"),
        Default::default(),
    );

    let wallpaper_path_clone = wallpaper_path.clone();

    application.connect_activate(move |app| {
        // Initialize global styles
        archvnde_common::init_theme();

        // Inject dynamic wallpaper background CSS globally for all lock windows
        let provider = gtk4::CssProvider::new();
        let custom_css = format!(
            ".lock-window {{ background-image: url('file://{}'); background-size: cover; background-position: center; }}",
            wallpaper_path_clone
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
    });

    application.run_with_args::<&str>(&[]);
}
