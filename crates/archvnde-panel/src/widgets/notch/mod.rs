use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn run_playerctl(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("playerctl")
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            return Some(stdout);
        }
    }
    None
}

fn decode_uri(uri: &str) -> String {
    let mut decoded = String::new();
    let mut chars = uri.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                if let Some(hex) = u8::from_str_radix(&format!("{}{}", h1, h2), 16).ok() {
                    decoded.push(hex as char);
                    continue;
                }
            }
        }
        decoded.push(c);
    }
    decoded
}

fn load_album_art(art_url: &str, size: i32) -> Option<gtk4::Image> {
    let path_str = art_url.strip_prefix("file://")?;
    let decoded_path = decode_uri(path_str);
    
    let pb = gdk_pixbuf::Pixbuf::from_file_at_scale(
        &decoded_path,
        size,
        size,
        true,
    ).ok()?;
    
    let texture = gdk4::Texture::for_pixbuf(&pb);
    Some(gtk4::Image::from_paintable(Some(&texture)))
}

/// Creates the macOS style dropdown notch in the panel center containing a music player.
pub fn create_system_notch() -> gtk4::Box {
    let notch_capsule = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    notch_capsule.add_css_class("panel-notch");
    notch_capsule.set_valign(gtk4::Align::Center);
    notch_capsule.set_halign(gtk4::Align::Center);

    // Notch content box (so we can transition opacity of contents)
    let notch_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    notch_content.add_css_class("notch-content");
    notch_content.set_valign(gtk4::Align::Center);
    notch_content.set_halign(gtk4::Align::Center);

    // --- 1. Default View (compact Dynamic Island) ---
    let default_view = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    default_view.set_valign(gtk4::Align::Center);
    default_view.set_halign(gtk4::Align::Center);
    let default_icon = archvnde_common::icon::get_icon("logo", 12);
    default_view.append(&default_icon);
    notch_content.append(&default_view);

    // --- 2. Music View (expanded Dynamic Island) ---
    let music_view = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    music_view.set_valign(gtk4::Align::Center);
    music_view.set_halign(gtk4::Align::Center);
    music_view.set_visible(false); // Hidden by default

    // Album Art container
    let art_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    art_container.set_valign(gtk4::Align::Center);
    let fallback_icon = archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6");
    fallback_icon.add_css_class("notch-album-art");
    art_container.append(&fallback_icon);

    // Track details
    let track_label = gtk4::Label::new(Some("No media"));
    track_label.add_css_class("notch-player-text");

    // Music Visualizer animation
    let visualizer_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    visualizer_box.add_css_class("notch-visualizer");
    visualizer_box.set_valign(gtk4::Align::Center);

    let mut bars = Vec::new();
    for _ in 0..4 {
        let bar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        bar.add_css_class("visualizer-bar");
        bar.set_size_request(2, 2);
        bar.set_valign(gtk4::Align::End);
        visualizer_box.append(&bar);
        bars.push(bar);
    }

    music_view.append(&art_container);
    music_view.append(&track_label);
    music_view.append(&visualizer_box);
    notch_content.append(&music_view);

    notch_capsule.append(&notch_content);

    // State variables
    let is_playing_state = Rc::new(Cell::new(false));
    let last_art_url = Rc::new(RefCell::new(String::new()));

    // Sine wave visualizer loop
    let bars_clone = bars.clone();
    let is_playing_state_clone = is_playing_state.clone();
    let mut step = 0;
    glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
        if is_playing_state_clone.get() {
            step += 1;
            for (i, bar) in bars_clone.iter().enumerate() {
                let val = (((step + i * 3) as f64 * 0.8).sin() * 5.0 + 7.0) as i32;
                bar.set_size_request(2, val.max(2).min(12));
            }
        } else {
            for bar in &bars_clone {
                bar.set_size_request(2, 2);
            }
        }
        glib::ControlFlow::Continue
    });

    // Playerctl polling loop
    let is_playing_state_clone2 = is_playing_state.clone();
    let last_art_url_clone = last_art_url.clone();
    let notch_capsule_clone = notch_capsule.clone();
    let default_view_clone = default_view.clone();
    let music_view_clone = music_view.clone();
    let track_label_clone = track_label.clone();
    let art_container_clone = art_container.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let status = run_playerctl(&["status"]);
        let playing = status.as_deref() == Some("Playing");
        is_playing_state_clone2.set(playing);

        if playing {
            // Get title & artist
            let title = run_playerctl(&["metadata", "title"]).unwrap_or_else(|| "Unknown Title".to_string());
            let artist = run_playerctl(&["metadata", "artist"]).unwrap_or_else(|| "Unknown Artist".to_string());
            
            // Format label: "Artist - Title" or just "Title"
            let label_text = if artist.is_empty() {
                title
            } else {
                format!("{} - {}", artist, title)
            };
            
            // Truncate to avoid overflowing Dynamic Island
            let display_text = if label_text.chars().count() > 18 {
                let truncated: String = label_text.chars().take(15).collect();
                format!("{}...", truncated)
            } else {
                label_text
            };
            track_label_clone.set_text(&display_text);

            // Check art url
            let art_url = run_playerctl(&["metadata", "mpris:artUrl"]).unwrap_or_default();
            let mut last_url = last_art_url_clone.borrow_mut();
            if *last_url != art_url {
                *last_url = art_url.clone();
                // Clear old art
                if let Some(child) = art_container_clone.first_child() {
                    art_container_clone.remove(&child);
                }
                
                // Load new art
                let new_art = if !art_url.is_empty() {
                    load_album_art(&art_url, 16).unwrap_or_else(|| {
                        archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6")
                    })
                } else {
                    archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6")
                };
                new_art.add_css_class("notch-album-art");
                art_container_clone.append(&new_art);
            }

            default_view_clone.set_visible(false);
            music_view_clone.set_visible(true);
            notch_capsule_clone.add_css_class("active-music");
        } else {
            default_view_clone.set_visible(true);
            music_view_clone.set_visible(false);
            notch_capsule_clone.remove_css_class("active-music");
        }

        glib::ControlFlow::Continue
    });

    notch_capsule
}
