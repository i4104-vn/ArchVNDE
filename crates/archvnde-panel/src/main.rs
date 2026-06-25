mod widgets;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use widgets::panel::create_status_indicators;
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
        let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
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

        // Notch Capsule (Apple MacOS style dropdown notch)
        let notch_capsule = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        notch_capsule.add_css_class("panel-notch");
        notch_capsule.set_valign(gtk4::Align::Start);
        notch_capsule.set_halign(gtk4::Align::Center);

        // Notch content box (so we can transition opacity of contents)
        let notch_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        notch_content.add_css_class("notch-content");
        notch_content.set_valign(gtk4::Align::Center);
        notch_content.set_halign(gtk4::Align::Center);

        // Search shortcut inside notch
        let notch_search_btn = gtk4::Button::new();
        notch_search_btn.add_css_class("notch-btn");
        let search_icon = archvnde_common::icon::get_icon("search", 14);
        notch_search_btn.set_child(Some(&search_icon));
        notch_search_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("archvnde-launcher").spawn();
        });

        // Sleek separator
        let notch_sep = gtk4::Label::new(Some("│"));
        notch_sep.add_css_class("notch-separator");

        // Media controller mock
        let notch_media_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        notch_media_box.set_valign(gtk4::Align::Center);
        
        let play_icon = archvnde_common::icon::get_icon("music", 14);
        play_icon.add_css_class("notch-media-icon");
        
        let media_label = gtk4::Label::new(Some("ArchVNDE OS"));
        media_label.add_css_class("notch-media-text");
        
        notch_media_box.append(&play_icon);
        notch_media_box.append(&media_label);

        // Quick lock button inside notch
        let notch_power_btn = gtk4::Button::new();
        notch_power_btn.add_css_class("notch-btn");
        let lock_icon = archvnde_common::icon::get_icon("lock", 14);
        notch_power_btn.set_child(Some(&lock_icon));
        
        notch_power_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("swaylock").spawn()
                .or_else(|_| std::process::Command::new("waylock").spawn());
        });

        notch_content.append(&notch_search_btn);
        notch_content.append(&notch_sep);
        notch_content.append(&notch_media_box);
        notch_content.append(&notch_power_btn);

        notch_capsule.append(&notch_content);
        center_box.append(&notch_capsule);

        // Right-aligned section: Status & Clock capsule
        let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        right_box.set_hexpand(true);
        right_box.set_halign(gtk4::Align::End);
        right_box.set_valign(gtk4::Align::Center);
        right_box.append(&status_indicators);

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
