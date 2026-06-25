mod widgets;

use gtk4::prelude::*;
use widgets::build_menu_ui;

fn main() {
    println!("Starting ArchVNDE Context Menu...");

    let application = gtk4::Application::new(
        Some("org.archvnde.menu"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        // Build menu window layout (from ui module)
        let window = build_menu_ui(app);

        // Present window
        window.present();
    });

    application.run();
}
