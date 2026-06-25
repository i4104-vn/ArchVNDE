#[path = "../widgets/mod.rs"]
mod widgets;

use gtk4::prelude::*;
use widgets::notch::create_system_notch;
use archvnde_common::{IslandState, update_island_state, clear_island_state};

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
            .default_width(450)
            .default_height(250)
            .build();

        // Create a layout container
        let box_layout = gtk4::CenterBox::new();
        box_layout.add_css_class("panel-box");

        // Create the notch capsule containing notch + automatic badge
        let notch = create_system_notch();
        box_layout.set_center_widget(Some(&notch));

        window.set_child(Some(&box_layout));
        window.present();

        // --- Demo Timeline ---

        // 1. Mock music playing immediately
        let _ = update_island_state(&IslandState {
            active: true,
            title: "ArchVNDE Radio".to_string(),
            subtitle: "Live Stream".to_string(),
            icon: "music".to_string(),
        });

        // 2. Trigger notification popup after 3 seconds (badge will drop down automatically!)
        glib::timeout_add_local(std::time::Duration::from_millis(3000), move || {
            let _ = update_island_state(&IslandState {
                active: true,
                title: "Update Available".to_string(),
                subtitle: "New version v1.2 ready to install".to_string(),
                icon: "bell".to_string(),
            });
            glib::ControlFlow::Break
        });

        // 3. Clear notification after 7 seconds
        glib::timeout_add_local(std::time::Duration::from_millis(7000), move || {
            let _ = clear_island_state();
            glib::ControlFlow::Break
        });
    });

    application.run();
}
