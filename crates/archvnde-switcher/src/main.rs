use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use archvnde_common::desktop::DesktopApp;

fn get_running_apps() -> Vec<DesktopApp> {
    let desktop_apps = archvnde_common::desktop::find_desktop_apps();
    let mut running = Vec::new();
    let mut detected_names = std::collections::HashSet::new();

    // 1. Get all running process names from /proc
    let mut running_processes = std::collections::HashSet::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.chars().all(|c| c.is_ascii_digit()) {
                        let comm_path = entry.path().join("comm");
                        if let Ok(comm) = std::fs::read_to_string(comm_path) {
                            let process_name = comm.trim().to_lowercase();
                            if !process_name.is_empty() {
                                running_processes.insert(process_name);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Match running processes with desktop entries
    for app in desktop_apps {
        let exec_parts: Vec<&str> = app.exec.split_whitespace().collect();
        if exec_parts.is_empty() {
            continue;
        }
        let exec_path = std::path::Path::new(exec_parts[0]);
        let exec_name = exec_path.file_name()
            .map(|f| f.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if exec_name.is_empty() {
            continue;
        }

        if running_processes.contains(&exec_name) {
            let app_key = exec_name.clone();
            if !detected_names.contains(&app_key) {
                detected_names.insert(app_key);
                running.push(app);
            }
        }
    }

    running
}

fn main() {
    println!("Starting ArchVNDE Alt-Tab Switcher...");

    let application = gtk4::Application::new(
        Some("org.archvnde.switcher"),
        Default::default(),
    );

    application.connect_activate(|app| {
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        // Center on screen
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.add_css_class("switcher-window");

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.add_css_class("switcher-box");

        let apps = get_running_apps();

        if apps.is_empty() {
            // Display "No apps running" state
            main_box.set_spacing(16);
            main_box.set_margin_top(30);
            main_box.set_margin_bottom(30);
            main_box.set_margin_start(50);
            main_box.set_margin_end(50);
            main_box.set_halign(gtk4::Align::Center);
            main_box.set_valign(gtk4::Align::Center);

            let no_apps_icon = archvnde_common::icon::get_system_or_file_icon("application-x-executable", "application-x-executable");
            no_apps_icon.set_pixel_size(48);
            no_apps_icon.set_halign(gtk4::Align::Center);

            let no_apps_lbl = gtk4::Label::new(Some("Không có ứng dụng nào đang chạy"));
            no_apps_lbl.add_css_class("switcher-app-title");
            no_apps_lbl.set_halign(gtk4::Align::Center);

            main_box.append(&no_apps_icon);
            main_box.append(&no_apps_lbl);

            window.set_child(Some(&main_box));

            let key_controller = gtk4::EventControllerKey::new();
            let window_close = window.clone();
            key_controller.connect_key_pressed(move |_, key, _, _| {
                match key {
                    gtk4::gdk::Key::Escape | gtk4::gdk::Key::Return => {
                        window_close.close();
                        gtk4::glib::Propagation::Stop
                    }
                    _ => gtk4::glib::Propagation::Proceed,
                }
            });
            window.add_controller(key_controller);
            window.present();
            return;
        }

        // 1. App Preview Stack
        let preview_stack = gtk4::Stack::new();
        preview_stack.add_css_class("preview-frame");
        preview_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        preview_stack.set_transition_duration(150);

        for app_item in &apps {
            let p_widget = create_app_preview(app_item);
            preview_stack.add_named(&p_widget, Some(app_item.name.as_str()));
        }
        main_box.append(&preview_stack);

        // 2. Selected App Details
        let details_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        details_box.add_css_class("switcher-details-box");
        details_box.set_halign(gtk4::Align::Center);

        let app_title_lbl = gtk4::Label::new(None);
        app_title_lbl.add_css_class("switcher-app-title");
        app_title_lbl.set_halign(gtk4::Align::Center);

        let app_sub_lbl = gtk4::Label::new(None);
        app_sub_lbl.add_css_class("switcher-app-subtitle");
        app_sub_lbl.set_halign(gtk4::Align::Center);

        details_box.append(&app_title_lbl);
        details_box.append(&app_sub_lbl);
        main_box.append(&details_box);

        // 3. Horizontal Icons Row
        let icons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        icons_row.add_css_class("switcher-list-row");
        icons_row.set_halign(gtk4::Align::Center);

        let mut item_buttons = Vec::new();

        for (idx, app_item) in apps.iter().enumerate() {
            let btn = gtk4::Button::new();
            btn.add_css_class("switcher-item-btn");
            
            let btn_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
            let app_icon_str = app_item.icon.as_deref().unwrap_or("application-x-executable");
            let icon_widget = archvnde_common::icon::get_system_or_file_icon(app_icon_str, "application-x-executable");
            icon_widget.set_pixel_size(32);
            icon_widget.add_css_class("switcher-item-icon");

            btn_box.append(&icon_widget);
            btn.set_child(Some(&btn_box));
            
            icons_row.append(&btn);
            item_buttons.push(btn);
        }
        main_box.append(&icons_row);

        window.set_child(Some(&main_box));

        // State tracking
        let current_index = Rc::new(RefCell::new(0));

        let update_selection = {
            let current_index = current_index.clone();
            let apps = apps.clone();
            let preview_stack = preview_stack.clone();
            let app_title_lbl = app_title_lbl.clone();
            let app_sub_lbl = app_sub_lbl.clone();
            let item_buttons = item_buttons.clone();

            move |new_idx: usize| {
                let mut idx = new_idx;
                if idx >= apps.len() {
                    idx = 0;
                }
                *current_index.borrow_mut() = idx;

                let app_item = &apps[idx];
                app_title_lbl.set_text(&app_item.name);
                app_sub_lbl.set_text(&format!("exec: {}", app_item.exec));

                preview_stack.set_visible_child_name(&app_item.name);

                for (i, btn) in item_buttons.iter().enumerate() {
                    if i == idx {
                        btn.add_css_class("selected");
                    } else {
                        btn.remove_css_class("selected");
                    }
                }
            }
        };

        // Initial selection setup
        let update_selection_rc = Rc::new(update_selection);
        update_selection_rc(0);

        // Click handlers on buttons
        for (i, btn) in item_buttons.iter().enumerate() {
            let update_sel = update_selection_rc.clone();
            btn.connect_clicked(move |_| {
                update_sel(i);
            });
        }

        // Keyboard navigation
        let key_controller = gtk4::EventControllerKey::new();
        let current_idx_key = current_index.clone();
        let update_sel_key = update_selection_rc.clone();
        let window_close = window.clone();
        let apps_key = apps.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let idx = *current_idx_key.borrow();
            match key {
                gtk4::gdk::Key::Tab | gtk4::gdk::Key::Right => {
                    let next = (idx + 1) % apps_key.len();
                    update_sel_key(next);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Left => {
                    let prev = if idx == 0 { apps_key.len() - 1 } else { idx - 1 };
                    update_sel_key(prev);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Return | gtk4::gdk::Key::space => {
                    let app_item = &apps_key[idx];
                    println!("Selected App: {}", app_item.name);
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Escape => {
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });
        window.add_controller(key_controller);

        window.present();
    });

    application.run();
}

fn create_app_preview(app: &DesktopApp) -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.add_css_class("preview-generic");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    card.add_css_class("preview-generic-box");
    card.set_halign(gtk4::Align::Center);
    card.set_valign(gtk4::Align::Center);

    let app_icon_str = app.icon.as_deref().unwrap_or("application-x-executable");
    let icon_w = archvnde_common::icon::get_system_or_file_icon(app_icon_str, "application-x-executable");
    icon_w.set_pixel_size(96);
    icon_w.set_halign(gtk4::Align::Center);

    let name_lbl = gtk4::Label::new(Some(&app.name));
    name_lbl.add_css_class("switcher-app-title");
    name_lbl.set_halign(gtk4::Align::Center);

    let desc_lbl = gtk4::Label::new(Some("Ứng dụng đang chạy"));
    desc_lbl.add_css_class("popup-time");
    desc_lbl.set_halign(gtk4::Align::Center);

    let command_lbl = gtk4::Label::new(Some(&format!("Lệnh: {}", app.exec)));
    command_lbl.add_css_class("popup-time");
    command_lbl.set_halign(gtk4::Align::Center);
    command_lbl.set_max_width_chars(40);
    command_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    card.append(&icon_w);
    card.append(&name_lbl);
    card.append(&desc_lbl);
    card.append(&command_lbl);

    main_box.append(&card);

    main_box.upcast::<gtk4::Widget>()
}
