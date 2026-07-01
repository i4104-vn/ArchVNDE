mod pam;
mod widgets;
mod render;

use gtk4::prelude::*;

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

    application.connect_activate(move |app| {
        // Initialize global styles
        archvnde_common::init_theme();

        // Build lock UI using render module
        render::build_lock_ui(app, &wallpaper_path);
    });

    application.run_with_args::<&str>(&[]);
}
