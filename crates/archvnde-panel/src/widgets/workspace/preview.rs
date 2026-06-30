use gtk4::prelude::*;
use archvnde_common::desktop::DesktopApp;
use super::apps::{focus_window, close_window};

/// Populates a Popover widget containing a vertical list of window titles grouped by app
pub fn populate_popover_previews(
    popover: &gtk4::Popover,
    windows: &[DesktopApp],
    app_id: &str,
) {
    let previews_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    previews_box.add_css_class("taskbar-previews-container");

    // Header
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    header.add_css_class("taskbar-previews-header");
    let header_label = gtk4::Label::new(Some("Tasks"));
    header_label.add_css_class("taskbar-previews-header-label");
    header.append(&header_label);
    previews_box.append(&header);

    // List of open windows
    for app in windows {
        let item_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        item_box.add_css_class("taskbar-preview-list-item-box");
        item_box.set_hexpand(true);

        let preview_btn = gtk4::Button::new();
        preview_btn.add_css_class("taskbar-preview-list-btn");
        preview_btn.set_hexpand(true);

        // Window title label
        let title_str = app.window_title.as_deref().unwrap_or("");
        let label_text = format!("● {}", if title_str.is_empty() {
            app.name.clone()
        } else {
            title_str.to_string()
        });

        let title_lbl = gtk4::Label::new(Some(&label_text));
        title_lbl.add_css_class("taskbar-preview-title");
        title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_lbl.set_max_width_chars(18);
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk4::Align::Start);

        preview_btn.set_child(Some(&title_lbl));

        // On click: focus the window and close popover
        let pop_close = popover.clone();
        let app_id_str = app.app_id.as_deref().unwrap_or(app_id);
        let app_id_str_clone = app_id_str.to_string();
        let title_str_clone = title_str.to_string();
        preview_btn.connect_clicked(move |_| {
            focus_window(&app_id_str_clone, &title_str_clone);
            pop_close.popdown();
        });

        // Dedicated window close/kill button
        let kill_btn = gtk4::Button::from_icon_name("window-close-symbolic");
        kill_btn.add_css_class("taskbar-preview-list-kill-btn");
        let pop_close_kill = popover.clone();
        let app_id_str_kill = app_id_str.to_string();
        let title_str_kill = title_str.to_string();
        kill_btn.connect_clicked(move |_| {
            close_window(&app_id_str_kill, &title_str_kill);
            pop_close_kill.popdown();
        });

        item_box.append(&preview_btn);
        item_box.append(&kill_btn);
        previews_box.append(&item_box);
    }

    // Separator and Action Buttons
    let (app_name, exec_cmd, icon_name) = if let Some(first_app) = windows.first() {
        (
            first_app.name.clone(),
            first_app.exec.clone(),
            first_app.icon.clone().unwrap_or_else(|| app_id.to_string()),
        )
    } else {
        (app_id.to_string(), app_id.to_string(), app_id.to_string())
    };

    // Divider (thanh divine)
    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    separator.add_css_class("taskbar-preview-separator");
    previews_box.append(&separator);

    // Button: <Tên của App> (Mở cửa sổ mới)
    let open_new_btn = gtk4::Button::new();
    open_new_btn.add_css_class("taskbar-preview-action-btn");
    open_new_btn.set_hexpand(true);

    let open_new_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    open_new_content.set_halign(gtk4::Align::Start);
    let open_new_icon = archvnde_common::icon::get_system_or_file_icon(&icon_name, "application-x-executable");
    open_new_icon.set_pixel_size(16);
    let open_new_label = gtk4::Label::new(Some(&app_name));
    open_new_label.add_css_class("taskbar-preview-action-label");
    open_new_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    open_new_label.set_max_width_chars(20);
    open_new_label.set_hexpand(true);
    open_new_label.set_halign(gtk4::Align::Start);
    open_new_content.append(&open_new_icon);
    open_new_content.append(&open_new_label);
    open_new_btn.set_child(Some(&open_new_content));

    let pop_open_new = popover.clone();
    let exec_cmd_clone = exec_cmd.clone();
    open_new_btn.connect_clicked(move |_| {
        let parts: Vec<&str> = exec_cmd_clone.split_whitespace().collect();
        if !parts.is_empty() {
            let program = parts[0];
            let args = &parts[1..];
            if let Err(e) = std::process::Command::new(program).args(args).spawn() {
                eprintln!("Failed to spawn command {}: {}", exec_cmd_clone, e);
            }
        }
        pop_open_new.popdown();
    });
    previews_box.append(&open_new_btn);

    // Button: Đóng tất cả
    let close_all_btn = gtk4::Button::new();
    close_all_btn.add_css_class("taskbar-preview-action-btn");
    close_all_btn.set_hexpand(true);

    let close_all_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    close_all_content.set_halign(gtk4::Align::Start);
    
    let close_all_icon = gtk4::Image::from_icon_name("window-close-symbolic");
    close_all_icon.set_pixel_size(16);
    close_all_icon.add_css_class("taskbar-preview-action-icon");

    let close_all_label = gtk4::Label::new(Some("Đóng tất cả"));
    close_all_label.add_css_class("taskbar-preview-action-label");
    close_all_content.append(&close_all_icon);
    close_all_content.append(&close_all_label);
    close_all_btn.set_child(Some(&close_all_content));

    let pop_close_all = popover.clone();
    let windows_clone: Vec<(String, String)> = windows
        .iter()
        .map(|app| {
            let app_id_str = app.app_id.as_deref().unwrap_or(app_id).to_string();
            let title_str = app.window_title.as_deref().unwrap_or("").to_string();
            (app_id_str, title_str)
        })
        .collect();

    close_all_btn.connect_clicked(move |_| {
        for (app_id_str, title_str) in &windows_clone {
            close_window(app_id_str, title_str);
        }
        pop_close_all.popdown();
    });
    previews_box.append(&close_all_btn);

    // Clear child on closed to free resources
    popover.connect_closed(|pop| {
        pop.set_child(None::<&gtk4::Widget>);
    });

    popover.set_child(Some(&previews_box));
}
