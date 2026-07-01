use crate::models::DesktopApp;
use gtk4::prelude::*;
use std::process::Command;

pub fn create_app_row(app: &DesktopApp, window: &gtk4::ApplicationWindow) -> gtk4::ListBoxRow {
    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    row_box.set_margin_start(6);
    row_box.set_margin_end(6);
    row_box.set_margin_top(6);
    row_box.set_margin_bottom(6);

    let icon_widget = archvnde_common::icon::get_system_or_file_icon(
        app.icon.as_deref().unwrap_or(""),
        "application-x-executable",
    );
    icon_widget.set_pixel_size(32);

    let name_label = gtk4::Label::new(Some(&app.name));
    name_label.set_xalign(0.0);

    row_box.append(&icon_widget);
    row_box.append(&name_label);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&row_box));
    
    let exec_cmd = app.exec.clone();
    let win_to_close = window.clone();
    row.connect_activate(move |_| {
        println!("Launching: {}", exec_cmd);
        let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let program = parts[0];
            let args = &parts[1..];
            if let Err(e) = Command::new(program).args(args).spawn() {
                eprintln!("Failed to spawn command {}: {}", exec_cmd, e);
            }
        }

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

    row
}
