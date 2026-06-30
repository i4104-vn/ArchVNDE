use gtk4::prelude::*;
use std::process::Command;
use super::file_search::{search_files, create_file_row};

pub fn populate_search_results(
    right_scroll: &gtk4::ScrolledWindow,
    query: &str,
    window: &gtk4::ApplicationWindow,
) {
    let query_trimmed = query.trim().to_string();
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
        right_scroll.set_child(Some(&right_content));
    } else {
        // Show the browser search button immediately (non-blocking)
        let browser_btn = gtk4::Button::new();
        browser_btn.add_css_class("launcher-browser-search-row");

        let browser_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        browser_box.set_valign(gtk4::Align::Center);

        let web_icon = archvnde_common::icon::get_system_or_file_icon("web-browser", "text-html");
        web_icon.set_pixel_size(20);
        web_icon.set_valign(gtk4::Align::Center);

        let search_fmt = archvnde_common::i18n::t("launcher.google_search");
        let browser_lbl = gtk4::Label::new(Some(&search_fmt.replace("{}", &query_trimmed)));
        browser_lbl.set_halign(gtk4::Align::Start);
        browser_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        browser_lbl.set_valign(gtk4::Align::Center);

        browser_box.append(&web_icon);
        browser_box.append(&browser_lbl);
        browser_btn.set_child(Some(&browser_box));

        let query_for_browser = query_trimmed.clone();
        let win_to_close = window.clone();
        browser_btn.connect_clicked(move |_| {
            let search_query = query_for_browser.replace(' ', "+");
            let url = format!("https://www.google.com/search?q={}", search_query);
            println!("Searching Google: {}", url);
            if let Err(e) = Command::new("xdg-open").arg(&url).spawn() {
                eprintln!("Failed to launch browser: {}", e);
            }
            win_to_close.close();
        });

        // Add a loading placeholder for file results
        let files_placeholder = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        right_content.append(&files_placeholder);
        right_content.append(&browser_btn);
        right_scroll.set_child(Some(&right_content));

        // Run file search in a background thread to avoid blocking the GTK main thread
        let query_for_search = query_trimmed.clone();
        let (sender, receiver) = std::sync::mpsc::channel::<Vec<std::path::PathBuf>>();

        std::thread::spawn(move || {
            let results = search_files(&query_for_search);
            let _ = sender.send(results);
        });

        // Poll the result from the background thread on the next idle cycle
        let files_placeholder_clone = files_placeholder.clone();
        let win_clone = window.clone();
        gtk4::glib::idle_add_local(move || {
            match receiver.try_recv() {
                Ok(matched_files) => {
                    if !matched_files.is_empty() {
                        let files_title = gtk4::Label::new(Some(&archvnde_common::i18n::t("launcher.files")));
                        files_title.add_css_class("launcher-section-title");
                        files_title.set_halign(gtk4::Align::Start);
                        files_placeholder_clone.append(&files_title);

                        for file_path in &matched_files {
                            let btn = create_file_row(file_path, &win_clone);
                            files_placeholder_clone.append(&btn);
                        }
                    }
                    gtk4::glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still waiting — try again next idle
                    gtk4::glib::ControlFlow::Continue
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    gtk4::glib::ControlFlow::Break
                }
            }
        });
    }
}

