use crate::models::DesktopApp;
use gtk4::prelude::*;
use std::process::Command;
use super::app_row::create_list_app_widget;
use super::file_search::{search_files, create_file_row};

pub fn populate_search_results(
    right_scroll: &gtk4::ScrolledWindow,
    query: &str,
    apps_rc: &[DesktopApp],
    window: &gtk4::ApplicationWindow,
) {
    let query_trimmed = query.trim();
    let right_content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    right_content.set_hexpand(true);
    right_content.set_vexpand(true);
    
    if query_trimmed.is_empty() {
        // If empty, display a placeholder guide or nothing
        let welcome_label = gtk4::Label::new(Some(&archvnde_common::i18n::t("launcher.welcome")));
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
            let apps_title = gtk4::Label::new(Some(&archvnde_common::i18n::t("launcher.apps")));
            apps_title.add_css_class("launcher-section-title");
            apps_title.set_halign(gtk4::Align::Start);
            right_content.append(&apps_title);
            
            let apps_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            for app in matched_apps.iter().take(5) {
                let btn = create_list_app_widget(app, window);
                apps_box.append(&btn);
            }
            right_content.append(&apps_box);
        }
        
        // Match Files
        let matched_files = search_files(query_trimmed);
        if !matched_files.is_empty() {
            let files_title = gtk4::Label::new(Some(&archvnde_common::i18n::t("launcher.files")));
            files_title.add_css_class("launcher-section-title");
            files_title.set_halign(gtk4::Align::Start);
            right_content.append(&files_title);
            
            let files_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            for file_path in &matched_files {
                let btn = create_file_row(file_path, window);
                files_box.append(&btn);
            }
            right_content.append(&files_box);
        }
        
        if matched_apps.is_empty() && matched_files.is_empty() {
            let no_results = gtk4::Label::new(Some(&archvnde_common::i18n::t("launcher.no_results")));
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
        
        let search_fmt = archvnde_common::i18n::t("launcher.google_search");
        let browser_lbl = gtk4::Label::new(Some(&search_fmt.replace("{}", query_trimmed)));
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
            
            win_to_close.close();
        });
        
        right_content.append(&browser_btn);
    }
    
    right_scroll.set_child(Some(&right_content));
}
