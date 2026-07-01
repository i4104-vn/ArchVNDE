//! Main entry point for the ArchVNDE Desktop Context Menu.
//! Initializes theme contexts and maps the popup menu window.

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
        archvnde_common::init_theme();
        let window = render::build_menu_ui(app);
        window.present();
    });

    application.run();
}

