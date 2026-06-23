use crate::core::{find_desktop_apps, DesktopApp};
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::process::Command;
use std::rc::Rc;

/// Configures and builds the GTK4 application launcher window overlay.
pub fn build_launcher_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    // Center on screen
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_default_size(450, 550);

    window.add_css_class("launcher-window");

    let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 15);
    box_layout.add_css_class("launcher-box");
    box_layout.set_margin_start(20);
    box_layout.set_margin_end(20);
    box_layout.set_margin_top(20);
    box_layout.set_margin_bottom(20);

    // Search Input
    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some("Search apps, files, settings..."));
    search_entry.add_css_class("launcher-search");

    // Application list container
    let scrolled_window = gtk4::ScrolledWindow::new();
    scrolled_window.set_vexpand(true);

    let list_box = gtk4::ListBox::new();
    list_box.add_css_class("launcher-list");

    // Load apps
    let apps = find_desktop_apps();
    let apps_rc = Rc::new(apps);

    // Helper function to populate the listbox
    let populate_list = {
        let list_box = list_box.clone();
        let window_clone = window.clone();
        move |filtered_apps: Vec<DesktopApp>| {
            // Clear existing
            while let Some(row) = list_box.first_child() {
                list_box.remove(&row);
            }

            for app in filtered_apps {
                let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
                row_box.set_margin_start(6);
                row_box.set_margin_end(6);
                row_box.set_margin_top(6);
                row_box.set_margin_bottom(6);

                // Try to load icon
                let icon_widget = if let Some(icon_name) = &app.icon {
                    if icon_name.starts_with('/') {
                        gtk4::Image::from_file(icon_name)
                    } else {
                        gtk4::Image::from_icon_name(icon_name)
                    }
                } else {
                    gtk4::Image::from_icon_name("application-x-executable")
                };
                icon_widget.set_pixel_size(32);

                let name_label = gtk4::Label::new(Some(&app.name));
                name_label.set_xalign(0.0);

                row_box.append(&icon_widget);
                row_box.append(&name_label);

                // Add to listbox row
                let row = gtk4::ListBoxRow::new();
                row.set_child(Some(&row_box));
                
                // Attach app meta data to execute on click
                let exec_cmd = app.exec.clone();
                let win_to_close = window_clone.clone();
                row.connect_activate(move |_| {
                    println!("Launching: {}", exec_cmd);
                    // Split command string into program and args
                    let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
                    if !parts.is_empty() {
                        let program = parts[0];
                        let args = &parts[1..];
                        if let Err(e) = Command::new(program).args(args).spawn() {
                            eprintln!("Failed to spawn command {}: {}", exec_cmd, e);
                        }
                    }
                    win_to_close.close();
                });

                list_box.append(&row);
            }
        }
    };

    // Populate initially
    populate_list(apps_rc.as_ref().clone());

    // Connect search bar text change
    let apps_search = apps_rc.clone();
    let populate_search = populate_list.clone();
    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_string().to_lowercase();
        let filtered: Vec<DesktopApp> = apps_search
            .iter()
            .filter(|app| app.name.to_lowercase().contains(&query))
            .cloned()
            .collect();
        populate_search(filtered);
    });

    // Close on escape key
    let key_controller = gtk4::EventControllerKey::new();
    let win_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            win_clone.close();
            gtk4::glib::Propagation::Proceed
        } else {
            gtk4::glib::Propagation::Stop
        }
    });
    window.add_controller(key_controller);

    scrolled_window.set_child(Some(&list_box));
    box_layout.append(&search_entry);
    box_layout.append(&scrolled_window);
    window.set_child(Some(&box_layout));

    // Animate inner layout
    archvnde_animation::slide_in(box_layout.upcast_ref(), archvnde_animation::SlideDirection::Down, 12, 240);

    window
}
