mod widgets;
mod render;

use gtk4::prelude::*;

fn main() {
    println!("Starting ArchVNDE Desktop Menu...");

    let application = gtk4::Application::new(
        Some("org.archvnde.menu"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        // Build menu window layout (from render module)
        let window = render::build_menu_ui(app);

        // Present window
        window.present();
    });

    application.run();
}
