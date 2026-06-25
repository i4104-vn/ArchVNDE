use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::playerctl::{load_album_art, run_playerctl};

pub fn start_player_polling_loop(
    is_playing_state: Rc<Cell<bool>>,
    notch_capsule: gtk4::Box,
    default_view: gtk4::Box,
    music_view: gtk4::Box,
    track_label: gtk4::Label,
    art_container: gtk4::Box,
) {
    let last_art_url = Rc::new(RefCell::new(String::new()));

    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let status = run_playerctl(&["status"]);
        let playing = status.as_deref() == Some("Playing");
        is_playing_state.set(playing);

        if playing {
            // Get title & artist
            let title = run_playerctl(&["metadata", "title"]).unwrap_or_else(|| "Unknown Title".to_string());
            let artist = run_playerctl(&["metadata", "artist"]).unwrap_or_else(|| "Unknown Artist".to_string());
            
            let label_text = if artist.is_empty() {
                title
            } else {
                format!("{} - {}", artist, title)
            };
            
            let display_text = if label_text.chars().count() > 18 {
                let truncated: String = label_text.chars().take(15).collect();
                format!("{}...", truncated)
            } else {
                label_text
            };
            track_label.set_text(&display_text);

            let art_url = run_playerctl(&["metadata", "mpris:artUrl"]).unwrap_or_default();
            let mut last_url = last_art_url.borrow_mut();
            if *last_url != art_url {
                *last_url = art_url.clone();
                
                if let Some(child) = art_container.first_child() {
                    art_container.remove(&child);
                }
                
                let new_art = if !art_url.is_empty() {
                    load_album_art(&art_url, 16).unwrap_or_else(|| {
                        archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6")
                    })
                } else {
                    archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6")
                };
                new_art.add_css_class("notch-album-art");
                art_container.append(&new_art);
            }

            default_view.set_visible(false);
            music_view.set_visible(true);
            notch_capsule.add_css_class("active-music");
        } else {
            default_view.set_visible(true);
            music_view.set_visible(false);
            notch_capsule.remove_css_class("active-music");
        }

        glib::ControlFlow::Continue
    });
}
