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
        // 1. Read custom island state from the TOML file
        let island_path = archvnde_common::island::get_island_state_path();
        let mut custom_state = None;
        if island_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&island_path) {
                let mut active = false;
                let mut title = String::new();
                let mut subtitle = String::new();
                let mut icon = String::new();
                for line in content.lines() {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let val = parts[1].trim().trim_matches('"');
                        match key {
                            "active" => active = val == "true",
                            "title" => title = val.to_string(),
                            "subtitle" => subtitle = val.to_string(),
                            "icon" => icon = val.to_string(),
                            _ => {}
                        }
                    }
                }
                custom_state = Some(archvnde_common::IslandState {
                    active,
                    title,
                    subtitle,
                    icon,
                });
            }
        }

        // 2. Read playerctl music state
        let status = run_playerctl(&["status"]);
        let playing = status.as_deref() == Some("Playing");

        // Determine if we should show the notch capsule
        let custom_active = custom_state.as_ref().map(|s| s.active).unwrap_or(false);
        let show_capsule = playing || custom_active;

        if show_capsule {
            // Update global playing state for visualizer animation
            is_playing_state.set(playing && !custom_active);

            // Render view depending on priority (Custom notifications take priority!)
            if custom_active {
                if let Some(state) = custom_state {
                    // Update track label with Custom Title + Subtitle
                    let display_text = if state.subtitle.is_empty() {
                        state.title
                    } else {
                        format!("{} - {}", state.title, state.subtitle)
                    };
                    let display_text = if display_text.chars().count() > 18 {
                        let truncated: String = display_text.chars().take(15).collect();
                        format!("{}...", truncated)
                    } else {
                        display_text
                    };
                    track_label.set_text(&display_text);

                    // Clear old art/icon
                    if let Some(child) = art_container.first_child() {
                        art_container.remove(&child);
                    }
                    
                    // Show custom icon (e.g. bell, download, shield, etc.)
                    let icon_name = if state.icon.is_empty() { "bell" } else { &state.icon };
                    let custom_icon = archvnde_common::icon::get_icon_colored(icon_name, 14, "#3b82f6");
                    custom_icon.add_css_class("notch-album-art");
                    art_container.append(&custom_icon);
                }
            } else {
                // Regular music player rendering
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

                // Check art url
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
            }

            default_view.set_visible(false);
            music_view.set_visible(true);
            notch_capsule.add_css_class("active-music");
            notch_capsule.set_visible(true);
        } else {
            is_playing_state.set(false);
            notch_capsule.remove_css_class("active-music");
            notch_capsule.set_visible(false);
        }

        glib::ControlFlow::Continue
    });
}
