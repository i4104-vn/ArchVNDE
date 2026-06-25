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

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu_box.add_css_class("dock-menu-box");

    let unpin_btn = gtk4::Button::with_label("Unpin from Dock");
    unpin_btn.add_css_class("menu-item-btn");

    let popover_clone = popover.clone();
    let config_clone = config.clone();
    let app_info_clone = app_info.clone();
    let dock_box_clone2 = dock_box_clone.clone();
    unpin_btn.connect_clicked(move |_| {
        popover_clone.popdown();
        let mut cfg = config_clone.borrow_mut();
        cfg.pinned_apps.retain(|a| a.command != app_info_clone.command);
        let _ = save_dock_config(&cfg);
        
        rebuild_dock_content(&dock_box_clone2, config_clone.clone());
    });

    menu_box.append(&unpin_btn);
    popover.set_child(Some(&menu_box));

    let gesture = gtk4::GestureClick::builder().button(3).build();
    let popover_clone2 = popover.clone();
    gesture.connect_released(move |_, _, _, _| {
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

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu_box.add_css_class("dock-menu-box");

    let available_apps = vec![
        PinnedApp {
            name: "Terminal (Foot)".to_string(),
            icon: "terminal".to_string(),
            command: "foot".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "Terminal (Alacritty)".to_string(),
            icon: "terminal".to_string(),
            command: "alacritty".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "Files (PCManFM)".to_string(),
            icon: "folder".to_string(),
            command: "pcmanfm".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "Files (Thunar)".to_string(),
            icon: "folder".to_string(),
            command: "thunar".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "Web Browser (Firefox)".to_string(),
            icon: "search".to_string(),
            command: "firefox".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "Web Browser (Chromium)".to_string(),
            icon: "search".to_string(),
            command: "chromium".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "Music Player (Amberol)".to_string(),
            icon: "music".to_string(),
            command: "amberol".to_string(),
            args: vec![],
        },
        PinnedApp {
            name: "System Settings".to_string(),
            icon: "settings".to_string(),
            command: "gnome-control-center".to_string(),
            args: vec![],
        },
    ];

    let popover_clone = popover.clone();
    let config_clone = config.clone();
    let dock_box_clone = dock_box.clone();

    btn.connect_clicked(move |_| {
        // Clear old menu items
        while let Some(child) = menu_box.first_child() {
            menu_box.remove(&child);
        }

        let current_pinned_commands: Vec<String> = config_clone.borrow().pinned_apps.iter().map(|a| a.command.clone()).collect();
        let mut count = 0;

        for app in &available_apps {
            if !current_pinned_commands.contains(&app.command) {
                let app_btn = gtk4::Button::with_label(&app.name);
                app_btn.add_css_class("menu-item-btn");
                
                let pop_clone = popover_clone.clone();
                let cfg_clone = config_clone.clone();
                let db_clone = dock_box_clone.clone();
                let app_clone = app.clone();
                app_btn.connect_clicked(move |_| {
                    pop_clone.popdown();
                    let mut cfg = cfg_clone.borrow_mut();
                    cfg.pinned_apps.push(app_clone.clone());
                    let _ = save_dock_config(&cfg);

                    rebuild_dock_content(&db_clone, cfg_clone.clone());
                });
                menu_box.append(&app_btn);
                count += 1;
            }
        }

        if count == 0 {
            let label = gtk4::Label::new(Some("All apps pinned"));
            label.add_css_class("menu-item-btn");
            menu_box.append(&label);
        }

        popover_clone.popup();
    });

    popover.set_child(Some(&menu_box));

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
