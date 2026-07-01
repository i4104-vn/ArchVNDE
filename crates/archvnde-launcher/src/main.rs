mod core;
mod widgets;

use gtk4::prelude::*;
use widgets::build_launcher_ui;

fn main() {
    println!("Starting ArchVNDE Launcher...");

    let application = gtk4::Application::new(
        Some("org.archvnde.launcher"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Toggle behavior: if window is already open, close it and return
        let windows = app.windows();
        if !windows.is_empty() {
            for win in windows {
                win.close();
            }
            return;
        }

        // Initialize style provider
        archvnde_common::init_theme();

        // Build launcher window layout (from ui module)
        let dummy_ref = std::rc::Rc::new(std::cell::RefCell::new(None));
        let window = build_launcher_ui(app, dummy_ref);

        // Present window
        window.present();
    });

    application.run();
}
