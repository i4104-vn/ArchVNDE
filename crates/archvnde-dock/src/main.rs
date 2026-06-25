mod config;
mod ui;

use gtk4::prelude::*;
use ui::build_dock_ui;

fn main() {
    println!("Starting ArchVNDE Dock...");

    let application = gtk4::Application::new(
        Some("org.archvnde.dock"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        // Build dock window layout (from ui module)
        let window = build_dock_ui(app);

        // Present window
        window.present();
    });

    application.run();
}
