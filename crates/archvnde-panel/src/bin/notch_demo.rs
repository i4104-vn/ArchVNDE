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

        // Center vertical layout to stack notch and notification badge
        let center_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        center_vbox.set_valign(gtk4::Align::Start);
        center_vbox.set_halign(gtk4::Align::Center);

        // Create the notch capsule
        let notch = create_system_notch();
        center_vbox.append(&notch);

        // Create the notification badge widget (drops down below notch)
        let notification_badge = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        notification_badge.add_css_class("island-badge");
        notification_badge.set_valign(gtk4::Align::Start);
        notification_badge.set_halign(gtk4::Align::Center);
        notification_badge.set_visible(false); // Hidden by default

        let badge_icon = archvnde_common::icon::get_icon_colored("bell", 14, "#3b82f6");
        let badge_text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        let badge_title = gtk4::Label::new(Some("System Update"));
        badge_title.add_css_class("badge-title");
        badge_title.set_halign(gtk4::Align::Start);
        let badge_desc = gtk4::Label::new(Some("Welcome to ArchVNDE Dynamic Island!"));
        badge_desc.add_css_class("badge-desc");
        badge_desc.set_halign(gtk4::Align::Start);
        
        badge_text_box.append(&badge_title);
        badge_text_box.append(&badge_desc);

        notification_badge.append(&badge_icon);
        notification_badge.append(&badge_text_box);
        center_vbox.append(&notification_badge);

        box_layout.set_center_widget(Some(&center_vbox));
        window.set_child(Some(&box_layout));
        window.present();

        // --- Demo Step 1: Mock music immediately ---
        let _ = update_island_state(&IslandState {
            active: true,
            title: "ArchVNDE Radio".to_string(),
            subtitle: "Live Stream".to_string(),
            icon: "music".to_string(),
        });

        // --- Demo Step 2: Trigger pop down notification after 3 seconds ---
        let badge_clone = notification_badge.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(3000), move || {
            // Update island state to system warning/bell notification
            let _ = update_island_state(&IslandState {
                active: true,
                title: "Update Available".to_string(),
                subtitle: "New version v1.2".to_string(),
                icon: "bell".to_string(),
            });

            // Make badge visible and trigger slide in animation
            badge_clone.set_visible(true);
            archvnde_common::animation::slide_in(
                badge_clone.clone().upcast_ref(),
                archvnde_common::animation::SlideDirection::Down,
                8,
                200,
            );
            glib::ControlFlow::Break
        });

        // --- Demo Step 3: Clear notification & hide badge after 7 seconds ---
        let badge_clone2 = notification_badge.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(7000), move || {
            badge_clone2.set_visible(false);
            let _ = clear_island_state();
            glib::ControlFlow::Break
        });
    });

    application.run();
}
