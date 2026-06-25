#[path = "../widgets/mod.rs"]
mod widgets;

use gtk4::prelude::*;
use widgets::notch::create_system_notch;

fn main() {
    println!("Starting Dynamic Island Notch Demo...");

    let application = gtk4::Application::new(
        Some("org.archvnde.notch.demo"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Dynamic Island Notch Demo")
            .default_width(400)
            .default_height(200)
            .build();

        // Create a layout container
        let box_layout = gtk4::CenterBox::new();
        box_layout.add_css_class("panel-box");

        // Create the notch capsule
        let notch = create_system_notch();
        box_layout.set_center_widget(Some(&notch));

        window.set_child(Some(&box_layout));
        window.present();
    });

    application.run();
}
