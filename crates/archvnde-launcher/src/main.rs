mod core;
mod ui;

use gtk4::prelude::*;
use ui::build_launcher_ui;

fn main() {
    println!("Starting ArchVNDE Launcher...");

    let application = gtk4::Application::new(
        Some("org.archvnde.launcher"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        // Build launcher window layout (from ui module)
        let window = build_launcher_ui(app);

        // Present window
        window.present();
    });

    application.run();
}
