use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardInteractivity, Layer, LayerShell};

fn main() {
    println!("Starting ArchVNDE Launcher...");

    let application = gtk4::Application::new(
        Some("org.archvnde.launcher"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Assign to Overlay layer to draw on top of everything
        window.set_layer(Layer::Overlay);

        // Require exclusive keyboard interactivity for searching
        window.set_keyboard_mode(KeyboardInteractivity::Exclusive);

        // Keep it floating in the center of the screen
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        // Set dimensions for the centered launcher box
        window.set_default_size(450, 550);

        window.add_css_class("launcher-window");

        let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 15);
        box_layout.set_margin_all(20);

        // Search Input
        let search_entry = gtk4::Entry::new();
        search_entry.set_placeholder_text(Some("Search apps, files, settings..."));
        search_entry.add_css_class("launcher-search");

        // Application list container
        let scrolled_window = gtk4::ScrolledWindow::new();
        scrolled_window.set_vexpand(true);

        let list_box = gtk4::ListBox::new();
        list_box.add_css_class("launcher-list");

        // Demo entries
        let apps = vec!["Alacritty", "Firefox Web Browser", "Files (Nautilus)", "VS Code", "Settings"];
        for app_name in apps {
            let label = gtk4::Label::new(Some(app_name));
            label.set_xalign(0.0);
            label.set_margin_all(8);
            list_box.append(&label);
        }

        scrolled_window.set_child(Some(&list_box));

        box_layout.append(&search_entry);
        box_layout.append(&scrolled_window);

        window.set_child(Some(&box_layout));

        // Present window with animation
        window.present();
        archvnde_animation::fade_in(&window, 300);
    });

    application.run();
}
