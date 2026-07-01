use crate::models::DesktopApp;
use gtk4::prelude::*;
use std::process::Command;
use std::cell::RefCell;
use std::rc::Rc;
use archvnde_common::models::{DockConfig, PinnedApp};
use archvnde_common::config::save_dock_config;

pub fn create_grid_app_widget(
    app: &DesktopApp,
    window: &gtk4::ApplicationWindow,
    config: Rc<RefCell<DockConfig>>,
    on_pinned_changed: Rc<dyn Fn()>,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("launcher-grid-item");
    btn.set_tooltip_text(Some(&app.name));

    let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    content_box.set_halign(gtk4::Align::Center);

    let icon_widget = archvnde_common::icon::get_system_or_file_icon(
        app.icon.as_deref().unwrap_or(""),
        "application-x-executable",
    );
    icon_widget.set_pixel_size(40);
    icon_widget.set_halign(gtk4::Align::Center);

    let name_label = gtk4::Label::new(Some(&app.name));
    name_label.set_halign(gtk4::Align::Center);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name_label.set_max_width_chars(10);
    name_label.add_css_class("launcher-grid-label");

    content_box.append(&icon_widget);
    content_box.append(&name_label);
    btn.set_child(Some(&content_box));

    // Click behavior (Launch)
    let exec_cmd = app.exec.clone();
    let win_to_close = window.clone();
    btn.connect_clicked(move |_| {
        println!("Launching from Grid: {}", exec_cmd);
        let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let program = parts[0];
            let args = &parts[1..];
            if let Err(e) = Command::new(program).args(args).spawn() {
                eprintln!("Failed to spawn command {}: {}", exec_cmd, e);
            }
        }

        // Close launcher with genie animation
        if let Some(child) = win_to_close.child() {
            if let Ok(box_layout) = child.downcast::<gtk4::Box>() {
                let win = win_to_close.clone();
                let w = box_layout.width().max(450);
                let h = box_layout.height().max(550);
                archvnde_common::animation::genie_out(
                    box_layout.upcast_ref(),
                    w,
                    h,
                    200,
                    move || {
                        win.close();
                    }
                );
                return;
            }
        }
        win_to_close.close();
    });

    // Right-click context menu (Pin/Unpin)
    let popover = gtk4::Popover::new();
    popover.set_parent(&btn);
    popover.set_has_arrow(true);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu_box.add_css_class("dock-menu-box");

    // Check if the app is currently pinned
    let is_pinned = config.borrow().pinned_apps.iter().any(|a| a.command == app.exec);

    let action_btn = if is_pinned {
        gtk4::Button::with_label("Unpin from Dock")
    } else {
        gtk4::Button::with_label("Pin to Dock")
    };
    action_btn.add_css_class("menu-item-btn");

    let popover_c = popover.clone();
    let config_c = config.clone();
    let app_c = app.clone();
    let on_pinned_changed_c = on_pinned_changed.clone();

    action_btn.connect_clicked(move |_| {
        popover_c.popdown();
        let mut cfg = config_c.borrow_mut();
        if is_pinned {
            cfg.pinned_apps.retain(|a| a.command != app_c.exec);
        } else {
            cfg.pinned_apps.push(PinnedApp {
                name: app_c.name.clone(),
                icon: app_c.icon.clone().unwrap_or_else(|| "application-x-executable".to_string()),
                command: app_c.exec.clone(),
                args: vec![],
            });
        }
        let _ = save_dock_config(&cfg);
        // Trigger grid reload to update pin/unpin visual state
        on_pinned_changed_c();
    });

    menu_box.append(&action_btn);
    popover.set_child(Some(&menu_box));

    let gesture = gtk4::GestureClick::builder().button(3).build();
    let popover_c2 = popover.clone();
    gesture.connect_released(move |_, _, _, _| {
        popover_c2.popup();
    });
    btn.add_controller(gesture);

    btn
}

pub fn create_list_app_widget(
    app: &DesktopApp,
    window: &gtk4::ApplicationWindow,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("launcher-list-item");
    btn.set_tooltip_text(Some(&app.name));

    let content_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    content_box.set_valign(gtk4::Align::Center);

    let icon_widget = archvnde_common::icon::get_system_or_file_icon(
        app.icon.as_deref().unwrap_or(""),
        "application-x-executable",
    );
    icon_widget.set_pixel_size(24);
    icon_widget.set_valign(gtk4::Align::Center);

    let name_label = gtk4::Label::new(Some(&app.name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name_label.set_max_width_chars(25);
    name_label.add_css_class("launcher-list-label");
    name_label.set_valign(gtk4::Align::Center);

    content_box.append(&icon_widget);
    content_box.append(&name_label);
    btn.set_child(Some(&content_box));

    let exec_cmd = app.exec.clone();
    let win_to_close = window.clone();
    btn.connect_clicked(move |_| {
        println!("Launching from List: {}", exec_cmd);
        let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let program = parts[0];
            let args = &parts[1..];
            if let Err(e) = Command::new(program).args(args).spawn() {
                eprintln!("Failed to spawn command {}: {}", exec_cmd, e);
            }
        }

        // Close launcher with genie animation
        if let Some(child) = win_to_close.child() {
            if let Ok(box_layout) = child.downcast::<gtk4::Box>() {
                let win = win_to_close.clone();
                let w = box_layout.width().max(450);
                let h = box_layout.height().max(550);
                archvnde_common::animation::genie_out(
                    box_layout.upcast_ref(),
                    w,
                    h,
                    200,
                    move || {
                        win.close();
                    }
                );
                return;
            }
        }
        win_to_close.close();
    });

    btn
}

