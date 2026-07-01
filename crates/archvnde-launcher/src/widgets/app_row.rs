use crate::models::DesktopApp;
use gtk4::prelude::*;
use std::process::Command;

pub fn create_grid_app_widget(
    app: &DesktopApp,
    window: &gtk4::ApplicationWindow,
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

        win_to_close.close();
    });

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

        win_to_close.close();
    });

    btn
}

