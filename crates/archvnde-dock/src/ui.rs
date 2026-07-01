use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use std::process::Command;

use crate::config::{load_dock_config, save_dock_config, PinnedApp, DockConfig};

fn create_dock_button(icon_name: &str, tooltip: &str, command: &str, args: &[&str]) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("dock-item-btn");
    btn.set_tooltip_text(Some(tooltip));

    // macOS style dock has larger icons (36px is clean and looks premium)
    let icon = archvnde_common::icon::get_icon_colored(icon_name, 36, "#ffffff");
    btn.set_child(Some(&icon));

    let cmd_str = command.to_string();
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    btn.connect_clicked(move |_| {
        let mut cmd = Command::new(&cmd_str);
        for arg in &args_vec {
            cmd.arg(arg);
        }
        let _ = cmd.spawn();
    });

    btn
}

fn attach_unpin_popover(
    btn: &gtk4::Button,
    app_info: PinnedApp,
    config: Rc<RefCell<DockConfig>>,
    dock_box_clone: gtk4::Box,
) {
    let popover = gtk4::Popover::new();
    popover.set_parent(btn);
    popover.set_has_arrow(true);

    let container_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    popover.set_child(Some(&container_box));

    // 1. Main menu view
    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu_box.add_css_class("dock-menu-box");

    let unpin_btn = gtk4::Button::with_label("Unpin from Dock");
    unpin_btn.add_css_class("menu-item-btn");
    let popover_c = popover.clone();
    let config_c = config.clone();
    let app_info_c = app_info.clone();
    let db_c = dock_box_clone.clone();
    unpin_btn.connect_clicked(move |_| {
        popover_c.popdown();
        let mut cfg = config_c.borrow_mut();
        cfg.pinned_apps.retain(|a| a.command != app_info_c.command);
        let _ = save_dock_config(&cfg);
        rebuild_dock_content(&db_c, config_c.clone());
    });
    menu_box.append(&unpin_btn);

    let change_icon_btn = gtk4::Button::with_label("Change Icon...");
    change_icon_btn.add_css_class("menu-item-btn");
    menu_box.append(&change_icon_btn);

    container_box.append(&menu_box);

    // 2. Edit View
    let edit_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    edit_box.add_css_class("dock-edit-popover");

    let edit_label = gtk4::Label::new(Some("Icon Name or Path:"));
    edit_label.add_css_class("dock-edit-label");
    edit_label.set_halign(gtk4::Align::Start);
    edit_box.append(&edit_label);

    let entry = gtk4::Entry::new();
    entry.add_css_class("dock-edit-entry");
    entry.set_text(&app_info.icon);
    edit_box.append(&entry);

    let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("menu-item-btn");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("menu-item-btn");
    actions_box.append(&cancel_btn);
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    actions_box.append(&spacer);
    actions_box.append(&save_btn);
    edit_box.append(&actions_box);

    // Navigation Wiring
    let container_box_c = container_box.clone();
    let menu_box_c = menu_box.clone();
    let edit_box_c = edit_box.clone();
    change_icon_btn.connect_clicked(move |_| {
        container_box_c.remove(&menu_box_c);
        container_box_c.append(&edit_box_c);
    });

    let popover_c2 = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_c2.popdown();
    });

    let popover_c3 = popover.clone();
    let config_c3 = config.clone();
    let app_info_c3 = app_info.clone();
    let db_c3 = dock_box_clone.clone();
    save_btn.connect_clicked(move |_| {
        popover_c3.popdown();
        let new_icon = entry.text().to_string();
        let mut cfg = config_c3.borrow_mut();
        if let Some(app) = cfg.pinned_apps.iter_mut().find(|a| a.command == app_info_c3.command) {
            app.icon = new_icon;
            let _ = save_dock_config(&cfg);
            rebuild_dock_content(&db_c3, config_c3.clone());
        }
    });

    // Right Click Controller
    let gesture = gtk4::GestureClick::builder().button(3).build();
    let popover_clone2 = popover.clone();
    let container_box_c3 = container_box.clone();
    let menu_box_c3 = menu_box.clone();
    let edit_box_c3 = edit_box.clone();
    gesture.connect_released(move |_, _, _, _| {
        // Reset to main menu
        if edit_box_c3.parent().is_some() {
            container_box_c3.remove(&edit_box_c3);
        }
        if menu_box_c3.parent().is_none() {
            container_box_c3.append(&menu_box_c3);
        }
        popover_clone2.popup();
    });
    btn.add_controller(gesture);
}

fn create_pin_app_button(
    config: Rc<RefCell<DockConfig>>,
    dock_box: gtk4::Box,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("dock-item-btn");
    btn.add_css_class("dock-add-btn");
    btn.set_tooltip_text(Some("Pin New App"));

    let icon = archvnde_common::icon::get_icon_colored("plus", 36, "#ffffff");
    btn.set_child(Some(&icon));

    let popover = gtk4::Popover::new();
    popover.set_parent(&btn);
    popover.set_has_arrow(true);

    let popover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    popover_box.add_css_class("dock-pin-popover-box");
    popover_box.set_size_request(260, 320);

    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some("Search system apps..."));
    search_entry.add_css_class("dock-pin-search");

<<<<<<< HEAD:crates/archvnde-dock/src/ui.rs
    let popover_clone = popover.clone();
    let config_clone = config.clone();
    let dock_box_clone = dock_box.clone();

    btn.connect_clicked(move |_| {
        // Clear old menu items
        while let Some(child) = menu_box.first_child() {
            menu_box.remove(&child);
        }
=======
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_vexpand(true);

    let list_box = gtk4::ListBox::new();
    list_box.add_css_class("dock-pin-list");
    scrolled.set_child(Some(&list_box));
>>>>>>> 9861cf1 (refactor: restructure workspace directories, modularize widgets, and implement macOS dock and Win11 launcher):crates/archvnde-dock/src/widgets/popovers.rs

    popover_box.append(&search_entry);
    popover_box.append(&scrolled);
    popover.set_child(Some(&popover_box));

    let all_apps = archvnde_common::desktop::find_desktop_apps();

    let populate_list = {
        let list_box = list_box.clone();
        let config_clone = config.clone();
        let dock_box_clone = dock_box.clone();
        let popover_clone = popover.clone();
        move |query: &str| {
            while let Some(child) = list_box.first_child() {
                list_box.remove(&child);
            }

            let current_pinned_commands: Vec<String> = config_clone
                .borrow()
                .pinned_apps
                .iter()
                .map(|a| a.command.clone())
                .collect();

            let query_lower = query.to_lowercase();
            let mut count = 0;
            for app in &all_apps {
                if current_pinned_commands.contains(&app.exec) {
                    continue;
                }
                if !query_lower.is_empty() && !app.name.to_lowercase().contains(&query_lower) {
                    continue;
                }

                let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                row_box.set_margin_start(4);
                row_box.set_margin_end(4);
                row_box.set_margin_top(4);
                row_box.set_margin_bottom(4);

                let icon = archvnde_common::icon::get_system_or_file_icon(
                    app.icon.as_deref().unwrap_or(""),
                    "application-x-executable",
                );
                icon.set_pixel_size(24);

                let label = gtk4::Label::new(Some(&app.name));
                label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

                row_box.append(&icon);
                row_box.append(&label);

                let row_btn = gtk4::Button::new();
                row_btn.add_css_class("menu-item-btn");
                row_btn.set_child(Some(&row_box));

                let pop_c = popover_clone.clone();
                let cfg_c = config_clone.clone();
                let db_c = dock_box_clone.clone();
                let app_c = app.clone();

                row_btn.connect_clicked(move |_| {
                    pop_c.popdown();
                    let mut cfg = cfg_c.borrow_mut();
                    cfg.pinned_apps.push(archvnde_common::models::PinnedApp {
                        name: app_c.name.clone(),
                        icon: app_c.icon.clone().unwrap_or_else(|| "application-x-executable".to_string()),
                        command: app_c.exec.clone(),
                        args: vec![],
                    });
                    let _ = save_dock_config(&cfg);
                    rebuild_dock_content(&db_c, cfg_c.clone());
                });
<<<<<<< HEAD:crates/archvnde-dock/src/ui.rs
                menu_box.append(&app_btn);
=======

                list_box.append(&row_btn);
>>>>>>> 9861cf1 (refactor: restructure workspace directories, modularize widgets, and implement macOS dock and Win11 launcher):crates/archvnde-dock/src/widgets/popovers.rs
                count += 1;
            }

<<<<<<< HEAD:crates/archvnde-dock/src/ui.rs
        if count == 0 {
            let label = gtk4::Label::new(Some("All apps pinned"));
            label.add_css_class("menu-item-btn");
            menu_box.append(&label);
=======
            if count == 0 {
                let label = gtk4::Label::new(Some("No applications found"));
                label.add_css_class("menu-item-btn");
                list_box.append(&label);
            }
>>>>>>> 9861cf1 (refactor: restructure workspace directories, modularize widgets, and implement macOS dock and Win11 launcher):crates/archvnde-dock/src/widgets/popovers.rs
        }
    };

    populate_list("");

    let populate_clone = populate_list.clone();
    search_entry.connect_changed(move |entry| {
        populate_clone(&entry.text());
    });

    btn.connect_clicked(move |_| {
        popover.popup();
    });

    btn
}

fn rebuild_dock_content(dock_box: &gtk4::Box, config: Rc<RefCell<DockConfig>>) {
    // Clear all existing widgets
    while let Some(child) = dock_box.first_child() {
        dock_box.remove(&child);
    }

    // 1. App Launcher (logo icon) - fixed
    let launcher_btn = create_dock_button("logo", "Application Launcher", "archvnde-launcher", &[]);
    dock_box.append(&launcher_btn);

    // Separator
    let sep1 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep1.add_css_class("dock-separator");
    dock_box.append(&sep1);

    // 2. Render pinned apps from configuration
    let pinned_apps = config.borrow().pinned_apps.clone();
    for app in pinned_apps {
        let args_ref: Vec<&str> = app.args.iter().map(|s| s.as_str()).collect();
        let btn = create_dock_button(&app.icon, &app.name, &app.command, &args_ref);
        
        attach_unpin_popover(&btn, app, config.clone(), dock_box.clone());
        dock_box.append(&btn);
    }

    // Separator
    let sep2 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep2.add_css_class("dock-separator");
    dock_box.append(&sep2);

    // 3. Pin App button (+)
    let pin_btn = create_pin_app_button(config.clone(), dock_box.clone());
    dock_box.append(&pin_btn);

    // Separator
    let sep3 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep3.add_css_class("dock-separator");
    dock_box.append(&sep3);

    // 4. Trash Bin - fixed
    let trash_btn = create_dock_button("trash", "Trash Bin", "pcmanfm", &["trash:///"]);
    dock_box.append(&trash_btn);
}

pub fn build_dock_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();

    // Assign to Top layer so it floats above normal windows
    window.set_layer(Layer::Top);

    // Set an exclusive zone of 64px so other windows respect the dock space at the bottom
    window.set_exclusive_zone(64);

    // Anchor ONLY to the bottom so it dynamically centers horizontally!
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Top, false);

    // Add margin to make it float above the bottom screen edge elegantly (macOS style)
    window.set_margin(Edge::Bottom, 10);

    window.add_css_class("dock-window");

    let dock_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    dock_box.add_css_class("dock-box");

    let config = Rc::new(RefCell::new(load_dock_config()));
    rebuild_dock_content(&dock_box, config);

    window.set_child(Some(&dock_box));

    window
}
