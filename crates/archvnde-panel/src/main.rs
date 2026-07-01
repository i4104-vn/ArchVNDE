mod widgets;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use widgets::panel::create_status_indicators;
use widgets::workspace::create_workspace_switcher;
use archvnde_island::create_system_island;

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

        // Define shared window states for mutual exclusivity
        let quick_settings_window = Rc::new(RefCell::new(None));
        let calendar_window = Rc::new(RefCell::new(None));

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Assign to the Top layer so it renders above normal windows
        window.set_layer(Layer::Top);

        // Set exclusive zone so other maximized windows don't overlap it
        window.set_exclusive_zone(36);

        // Anchor it to the top, left, and right edges of the screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Set default height of the panel
        window.set_default_size(0, 36);

        // Add styling class
        window.add_css_class("panel-window");

        // Layout container
        let box_layout = gtk4::CenterBox::new();
        box_layout.add_css_class("panel-box");

        // 1. Logo Button (launches launcher)
        let logo_btn = gtk4::Button::new();
        logo_btn.add_css_class("panel-logo-btn");
        let logo_icon = archvnde_common::icon::get_icon("logo", 16);
        logo_btn.set_child(Some(&logo_icon));
        logo_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("archvnde-launcher").spawn();
        });

        // 2. Workspace Switcher
        let workspace_box = create_workspace_switcher();

        // Create a separator to visual separate logo and dots inside the same capsule
        let separator = gtk4::Label::new(Some("│"));
        separator.add_css_class("capsule-separator");

        workspace_box.prepend(&separator);
        workspace_box.prepend(&logo_btn);

        // 3. Unified Status and Clock Capsule
        let status_indicators = create_status_indicators(
            app,
            quick_settings_window.clone(),
            calendar_window.clone(),
        );

        // Left-aligned section: Workspaces capsule (now containing logo + separator + dots)
        let left_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        left_box.set_hexpand(true);
        left_box.set_halign(gtk4::Align::Start);
        left_box.set_valign(gtk4::Align::Center);
        left_box.append(&workspace_box);
        // Center-aligned section: Clean placeholder center space with interactive notch
        let center_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        center_box.set_hexpand(true);
        center_box.set_halign(gtk4::Align::Center);
        center_box.set_valign(gtk4::Align::Start);

        let notch_capsule = create_system_island();
        center_box.append(&notch_capsule);

        // Right-aligned section: Status & Clock capsule
        let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        right_box.set_hexpand(true);
        right_box.set_halign(gtk4::Align::End);
        right_box.set_valign(gtk4::Align::Center);
        right_box.append(&status_indicators);

        // Assemble columns into the main panel box using CenterBox
        box_layout.set_start_widget(Some(&left_box));
        box_layout.set_center_widget(Some(&center_box));
        box_layout.set_end_widget(Some(&right_box));

        window.set_child(Some(&box_layout));

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
