use gtk4::prelude::*;
use std::process::Command;
use std::path::{Path, PathBuf};

pub fn search_files(query: &str) -> Vec<PathBuf> {
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

pub fn create_file_row(path: &Path, window: &gtk4::ApplicationWindow) -> gtk4::Button {
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
        
        win_to_close.close();
    });
    
    btn
}
