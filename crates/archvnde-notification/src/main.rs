use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::thread;

fn main() {
    println!("Starting ArchVNDE Notification Daemon...");

    // Spawn D-Bus handler thread using Tokio
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            println!("Notification D-Bus daemon thread started. Ready to listen to org.freedesktop.Notifications...");
            // Placeholder: sleep to keep the thread alive
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });
    });

    let application = gtk4::Application::new(
        Some("org.archvnde.notification"),
        Default::default(),
    );

    application.connect_activate(|app| {
        let window = gtk4::ApplicationWindow::new(app);

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Run in Overlay layer to draw on top, but without taking keyboard focus
        window.set_layer(Layer::Overlay);

        // Position it at the top-right corner of the screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        
        // Add margins so it doesn't touch the screen edge directly
        window.set_margin(Edge::Top, 15);
        window.set_margin(Edge::Right, 15);

        // Dimensions of the notification card
        window.set_default_size(320, 80);

        window.add_css_class("notification-card");

        let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
        box_layout.set_margin_all(12);

        let title_label = gtk4::Label::new(Some("System Notification"));
        title_label.set_xalign(0.0);
        title_label.add_css_class("notification-title");
        
        let body_label = gtk4::Label::new(Some("Welcome to ArchVNDE Custom Desktop Shell."));
        body_label.set_xalign(0.0);
        body_label.add_css_class("notification-body");

        box_layout.append(&title_label);
        box_layout.append(&body_label);

        window.set_child(Some(&box_layout));

        // Present window
        window.present();
    });

    application.run();
}
