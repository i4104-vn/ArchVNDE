use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn main() {
    // Set up logging or debug output
    println!("Starting ArchVNDE Panel...");

    let application = gtk4::Application::new(
        Some("org.archvnde.panel"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Assign to the Top layer so it renders above normal windows
        window.set_layer(Layer::Top);

        // Set exclusive zone so other maximized windows don't overlap it
        window.set_exclusive_zone(40);

        // Anchor it to the top, left, and right edges of the screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Set default height of the panel
        window.set_default_size(0, 40);

        // Add styling class
        window.add_css_class("panel-window");

        // Add placeholder UI widget
        let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        box_layout.set_margin_start(15);
        box_layout.set_margin_end(15);

        let title_label = gtk4::Label::new(Some("ArchVNDE"));
        title_label.add_css_class("panel-title");
        
        let center_label = gtk4::Label::new(Some("June 28, 10:33 PM"));
        center_label.set_hexpand(true);

        let status_label = gtk4::Label::new(Some("Wi-Fi | 100%"));

        box_layout.append(&title_label);
        box_layout.append(&center_label);
        box_layout.append(&status_label);

        window.set_child(Some(&box_layout));

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
