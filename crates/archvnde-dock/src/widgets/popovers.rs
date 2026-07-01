use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{save_dock_config, PinnedApp, DockConfig};
use super::rebuild_dock_content;

pub fn attach_unpin_popover(
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

pub fn create_pin_app_button(
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
    let menu_box_clone = menu_box.clone();

    btn.connect_clicked(move |_| {
        // Clear old menu items
        while let Some(child) = menu_box_clone.first_child() {
            menu_box_clone.remove(&child);
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
                menu_box_clone.append(&app_btn);
                count += 1;
            }
        }

        if count == 0 {
            let label = gtk4::Label::new(Some("All apps pinned"));
            label.add_css_class("menu-item-btn");
            menu_box_clone.append(&label);
        }

        popover_clone.popup();
    });

    popover.set_child(Some(&menu_box));

    btn
}
