pub mod app_row;
pub mod footer;

use crate::core::find_desktop_apps;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::rc::Rc;
use std::cell::RefCell;
use std::process::Command;
use footer::create_launcher_footer;
use app_row::create_list_app_widget;
use crate::models::DesktopApp;

fn search_files(query: &str) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    if query.trim().len() < 2 {
        return results;
    }
    let query_lower = query.to_lowercase();
    if let Some(home_dir) = dirs::home_dir() {
        let search_dirs = vec![
            home_dir.join("Desktop"),
            home_dir.join("Downloads"),
            home_dir.join("Documents"),
        ];
        
        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }
            let mut stack = vec![(dir, 0)];
            while let Some((current_dir, depth)) = stack.pop() {
                if results.len() >= 8 {
                    break;
                }
                if let Ok(entries) = std::fs::read_dir(current_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if file_name.starts_with('.') || 
                           file_name == "node_modules" || 
                           file_name == "target" || 
                           file_name == "build" || 
                           file_name == ".git" {
                            continue;
                        }
                        if file_name.to_lowercase().contains(&query_lower) {
                            results.push(path.clone());
                        }
                        if path.is_dir() && depth < 1 {
                            stack.push((path, depth + 1));
                        }
                    }
                }
            }
        }
    }
    results
}

fn create_file_row(path: &std::path::Path, window: &gtk4::ApplicationWindow) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("launcher-file-item");
    
    let path_str = path.to_string_lossy().to_string();
    btn.set_tooltip_text(Some(&path_str));
    
    let content_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    content_box.set_valign(gtk4::Align::Center);
    
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let icon_name = if path.is_dir() {
        "folder".to_string()
    } else {
        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image-x-generic".to_string(),
            "pdf" => "document-pdf".to_string(),
            "zip" | "tar" | "gz" | "xz" | "rar" | "7z" => "package-x-generic".to_string(),
            "mp3" | "wav" | "ogg" | "flac" => "audio-x-generic".to_string(),
            "mp4" | "mkv" | "avi" | "mov" => "video-x-generic".to_string(),
            "html" | "htm" | "css" | "js" | "ts" => "text-html".to_string(),
            _ => "text-x-generic".to_string(),
        }
    };
    
    let icon_widget = archvnde_common::icon::get_system_or_file_icon(&icon_name, "text-x-generic");
    icon_widget.set_pixel_size(20);
    icon_widget.set_valign(gtk4::Align::Center);
    
    let file_name_str = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let name_label = gtk4::Label::new(Some(&file_name_str));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name_label.set_max_width_chars(35);
    name_label.add_css_class("launcher-file-label");
    name_label.set_valign(gtk4::Align::Center);
    
    content_box.append(&icon_widget);
    content_box.append(&name_label);
    btn.set_child(Some(&content_box));
    
    let win_to_close = window.clone();
    btn.connect_clicked(move |_| {
        println!("Opening file: {}", path_str);
        if let Err(e) = Command::new("xdg-open").arg(&path_str).spawn() {
            eprintln!("Failed to open file {}: {}", path_str, e);
        }
        
        if let Some(child) = win_to_close.child() {
            if let Ok(box_layout) = child.downcast::<gtk4::Box>() {
                let win = win_to_close.clone();
                let w = box_layout.width().max(780);
                let h = box_layout.height().max(560);
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

pub fn build_launcher_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    
    archvnde_common::window::init_layer_window(
        &window,
        Layer::Overlay,
        KeyboardMode::Exclusive,
        -1,
        &[
            (Edge::Top, false),
            (Edge::Bottom, true),
            (Edge::Left, false),
            (Edge::Right, false),
        ],
        12,
    );

    window.set_default_size(780, 560);
    window.add_css_class("launcher-window");

    let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    box_layout.add_css_class("launcher-box");
    box_layout.set_margin_start(16);
    box_layout.set_margin_end(16);
    box_layout.set_margin_top(16);
    box_layout.set_margin_bottom(16);

    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some("Tìm ứng dụng hoặc tệp tin..."));
    search_entry.add_css_class("launcher-search");

    // Horizontal split box for two columns
    let columns_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    columns_box.add_css_class("launcher-columns-box");
    columns_box.set_vexpand(true);

    // Left column: scrollable all apps list (always shows all apps)
    let left_scroll = gtk4::ScrolledWindow::new();
    left_scroll.add_css_class("launcher-left-column");
    left_scroll.set_size_request(280, -1);
    left_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

    let left_list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let apps = find_desktop_apps();
    let apps_rc = Rc::new(apps);

    for app in apps_rc.iter() {
        let btn = create_list_app_widget(app, &window);
        left_list_box.append(&btn);
    }
    left_scroll.set_child(Some(&left_list_box));

    // Right column: dynamic search results
    let right_scroll = gtk4::ScrolledWindow::new();
    right_scroll.add_css_class("launcher-right-column");
    right_scroll.set_hexpand(true);
    right_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

    let current_query = Rc::new(RefCell::new(String::new()));

    let populate_impl = {
        let current_query = current_query.clone();
        let apps_rc = apps_rc.clone();
        let right_scroll = right_scroll.clone();
        let window = window.clone();

        move || {
            let query = current_query.borrow();
            let query_trimmed = query.trim();
            
            let right_content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
            right_content.set_hexpand(true);
            right_content.set_vexpand(true);
            
            if query_trimmed.is_empty() {
                // If empty, display a placeholder guide or nothing
                let welcome_label = gtk4::Label::new(Some("Nhập từ khóa để tìm kiếm ứng dụng và tệp tin..."));
                welcome_label.add_css_class("launcher-no-results");
                welcome_label.set_halign(gtk4::Align::Center);
                welcome_label.set_valign(gtk4::Align::Center);
                welcome_label.set_vexpand(true);
                welcome_label.set_hexpand(true);
                right_content.append(&welcome_label);
            } else {
                // Search results view: Apps, Files, Browser Search
                let query_lower = query_trimmed.to_lowercase();
                
                // Match Apps
                let matched_apps: Vec<DesktopApp> = apps_rc
                    .iter()
                    .filter(|app| app.name.to_lowercase().contains(&query_lower))
                    .cloned()
                    .collect();
                
                if !matched_apps.is_empty() {
                    let apps_title = gtk4::Label::new(Some("Ứng dụng"));
                    apps_title.add_css_class("launcher-section-title");
                    apps_title.set_halign(gtk4::Align::Start);
                    right_content.append(&apps_title);
                    
                    let apps_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
                    for app in matched_apps.iter().take(5) {
                        let btn = create_list_app_widget(app, &window);
                        apps_box.append(&btn);
                    }
                    right_content.append(&apps_box);
                }
                
                // Match Files
                let matched_files = search_files(query_trimmed);
                if !matched_files.is_empty() {
                    let files_title = gtk4::Label::new(Some("Tập tin"));
                    files_title.add_css_class("launcher-section-title");
                    files_title.set_halign(gtk4::Align::Start);
                    right_content.append(&files_title);
                    
                    let files_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
                    for file_path in matched_files {
                        let btn = create_file_row(&file_path, &window);
                        files_box.append(&btn);
                    }
                    right_content.append(&files_box);
                }
                
                if matched_apps.is_empty() && matched_files.is_empty() {
                    let no_results = gtk4::Label::new(Some("Không tìm thấy kết quả phù hợp"));
                    no_results.add_css_class("launcher-no-results");
                    no_results.set_halign(gtk4::Align::Center);
                    no_results.set_valign(gtk4::Align::Center);
                    no_results.set_vexpand(true);
                    right_content.append(&no_results);
                }
                
                // Search Browser Row
                let browser_btn = gtk4::Button::new();
                browser_btn.add_css_class("launcher-browser-search-row");
                
                let browser_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                browser_box.set_valign(gtk4::Align::Center);
                
                let web_icon = archvnde_common::icon::get_system_or_file_icon("web-browser", "text-html");
                web_icon.set_pixel_size(20);
                web_icon.set_valign(gtk4::Align::Center);
                
                let browser_lbl = gtk4::Label::new(Some(&format!("Tìm trên Google cho \"{}\"", query_trimmed)));
                browser_lbl.set_halign(gtk4::Align::Start);
                browser_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                browser_lbl.set_valign(gtk4::Align::Center);
                
                browser_box.append(&web_icon);
                browser_box.append(&browser_lbl);
                browser_btn.set_child(Some(&browser_box));
                
                let query_string = query_trimmed.to_string();
                let win_to_close = window.clone();
                browser_btn.connect_clicked(move |_| {
                    let search_query = query_string.replace(' ', "+");
                    let url = format!("https://www.google.com/search?q={}", search_query);
                    println!("Searching Google: {}", url);
                    if let Err(e) = Command::new("xdg-open").arg(&url).spawn() {
                        eprintln!("Failed to launch browser: {}", e);
                    }
                    
                    if let Some(child) = win_to_close.child() {
                        if let Ok(box_layout) = child.downcast::<gtk4::Box>() {
                            let win = win_to_close.clone();
                            let w = box_layout.width().max(780);
                            let h = box_layout.height().max(560);
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
                
                right_content.append(&browser_btn);
            }
            
            right_scroll.set_child(Some(&right_content));
        }
    };

    let populate_impl_rc = Rc::new(populate_impl);

    // Initial populate
    populate_impl_rc();

    let current_query_search = current_query.clone();
    let populate_grid_search = populate_impl_rc.clone();
    search_entry.connect_changed(move |entry| {
        *current_query_search.borrow_mut() = entry.text().to_string();
        populate_grid_search();
    });

    let is_animating = Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();
    let win_clone_close = window.clone();
    let box_layout_clone_close = box_layout.clone();
    window.connect_close_request(move |_| {
        if is_animating_clone.get() {
            return gtk4::glib::Propagation::Proceed;
        }
        is_animating_clone.set(true);
        let win_cb = win_clone_close.clone();
        let box_layout_cb = box_layout_clone_close.clone();
        let w = box_layout_cb.width().max(780);
        let h = box_layout_cb.height().max(560);
        archvnde_common::animation::genie_out(
            box_layout_cb.upcast_ref(),
            w,
            h,
            200,
            move || {
                win_cb.destroy();
            }
        );
        gtk4::glib::Propagation::Stop
    });

    window.connect_is_active_notify(|win| {
        if !win.is_active() {
            win.close();
        }
    });

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

    box_layout.append(&search_entry);
    
    columns_box.append(&left_scroll);
    columns_box.append(&right_scroll);
    box_layout.append(&columns_box);

    let footer_sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    footer_sep.add_css_class("launcher-footer-separator");
    box_layout.append(&footer_sep);

    let footer = create_launcher_footer();
    box_layout.append(&footer);

    window.set_child(Some(&box_layout));

    archvnde_common::animation::genie_in(box_layout.upcast_ref(), 780, 560, 240);

    window
}
