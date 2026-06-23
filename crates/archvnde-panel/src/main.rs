mod widgets;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use widgets::clock::create_clock_widget;
use widgets::quick_settings::create_settings_button;
use widgets::workspace::create_workspace_switcher;

fn main() {
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

        // Layout container
        let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        box_layout.add_css_class("panel-box");
        box_layout.set_margin_start(15);
        box_layout.set_margin_end(15);

        // 1. Logo Title Label
        let title_label = gtk4::Label::new(Some("ArchVNDE"));
        title_label.add_css_class("panel-title");

        // 2. Workspace Switcher (from widgets::workspace)
        let workspace_box = create_workspace_switcher();

        // 3. Clock Widget (from widgets::clock)
        let clock_label = create_clock_widget();

        // 4. Quick Settings Button (from widgets::quick_settings)
        let settings_button = create_settings_button(app);

        // Left-aligned section: Logo + Workspaces
        let left_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        left_box.set_hexpand(true);
        left_box.set_halign(gtk4::Align::Start);
        left_box.append(&title_label);
        left_box.append(&workspace_box);

        // Center-aligned section: Weather + Date & Time
        let center_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        center_box.set_hexpand(true);
        center_box.set_halign(gtk4::Align::Center);
        center_box.append(&clock_label);

        // Right-aligned section: Quick Settings system status trigger
        let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        right_box.set_hexpand(true);
        right_box.set_halign(gtk4::Align::End);
        right_box.append(&settings_button);

        // Assemble columns into the main panel box
        box_layout.append(&left_box);
        box_layout.append(&center_box);
        box_layout.append(&right_box);

        window.set_child(Some(&box_layout));

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
