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
    notification_badge: gtk4::Box,
    badge_title: gtk4::Label,
    badge_desc: gtk4::Label,
    badge_icon_container: gtk4::Box,

    // Popover widgets
    popover_title: gtk4::Label,
    popover_artist: gtk4::Label,
    popover_art_container: gtk4::Box,
    popover_app_name: gtk4::Label,
    play_img: gtk4::Image,
) {
    let last_art_url = Rc::new(RefCell::new(String::new()));
    let was_custom_active = Rc::new(Cell::new(false));

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
            is_playing_state.set(playing || (custom_active && custom_state.as_ref().map(|s| s.icon.as_str()) == Some("music")));

            // Render view depending on priority (Custom notifications take priority!)
            if custom_active {
                if let Some(state) = custom_state.as_ref() {
                    // Update track label with Custom Title + Subtitle
                    let display_text = if state.subtitle.is_empty() {
                        state.title.clone()
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

                // Handle Notification Badge slide-down
                if !was_custom_active.get() {
                    was_custom_active.set(true);
                    if let Some(state) = custom_state.as_ref() {
                        badge_title.set_text(&state.title);
                        badge_desc.set_text(&state.subtitle);

                        if let Some(child) = badge_icon_container.first_child() {
                            badge_icon_container.remove(&child);
                        }
                        let icon_name = if state.icon.is_empty() { "bell" } else { &state.icon };
                        let custom_icon = archvnde_common::icon::get_icon_colored(icon_name, 14, "#3b82f6");
                        badge_icon_container.append(&custom_icon);
                    }
                    notification_badge.set_visible(true);
                    archvnde_common::animation::slide_in(
                        notification_badge.clone().upcast_ref(),
                        archvnde_common::animation::SlideDirection::Down,
                        8,
                        200,
                    );
                }
            } else {
                // Regular music player rendering
                let title = run_playerctl(&["metadata", "title"]).unwrap_or_else(|| "Unknown Title".to_string());
                let artist = run_playerctl(&["metadata", "artist"]).unwrap_or_else(|| "Unknown Artist".to_string());

                let label_text = if artist.is_empty() {
                    title.clone()
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

                popover_title.set_text(&title);
                popover_artist.set_text(&artist);

                let player_name = run_playerctl(&["metadata", "--format", "{{ playerName }}"])
                    .map(|s| {
                        let mut chars = s.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .unwrap_or_else(|| "Music Player".to_string());
                popover_app_name.set_text(&player_name);

                let play_icon = if playing {
                    "media-playback-pause-symbolic"
                } else {
                    "media-playback-start-symbolic"
                };
                play_img.set_from_icon_name(Some(play_icon));

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

                    // Update Popover art container
                    if let Some(child) = popover_art_container.first_child() {
                        popover_art_container.remove(&child);
                    }
                    let new_large_art = if !art_url.is_empty() {
                        load_album_art(&art_url, 120).unwrap_or_else(|| {
                            archvnde_common::icon::get_icon_colored("music", 64, "#3b82f6")
                        })
                    } else {
                        archvnde_common::icon::get_icon_colored("music", 64, "#3b82f6")
                    };
                    new_large_art.add_css_class("media-popover-art");
                    new_large_art.set_size_request(180, 120);
                    popover_art_container.append(&new_large_art);
                }

                // Hide custom notification badge when custom alert is gone
                if was_custom_active.get() {
                    was_custom_active.set(false);
                    archvnde_common::animation::slide_out(
                        notification_badge.clone().upcast_ref(),
                        archvnde_common::animation::SlideDirection::Up,
                        8,
                        200,
                        true,
                    );
                }
            }

            default_view.set_visible(false);
            music_view.set_visible(true);
            if !notch_capsule.is_visible() {
                notch_capsule.add_css_class("active-music");
                archvnde_common::animation::zoom_in(
                    notch_capsule.clone().upcast_ref(),
                    200,
                    22,
                    300,
                );
            }
        } else {
            is_playing_state.set(false);
            if notch_capsule.is_visible() {
                let notch_capsule_clone = notch_capsule.clone();
                archvnde_common::animation::zoom_out(
                    notch_capsule.clone().upcast_ref(),
                    200,
                    300,
                    true,
                );
                glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                    notch_capsule_clone.remove_css_class("active-music");
                });
            }
            if was_custom_active.get() {
                was_custom_active.set(false);
                archvnde_common::animation::slide_out(
                    notification_badge.clone().upcast_ref(),
                    archvnde_common::animation::SlideDirection::Up,
                    8,
                    200,
                    true,
                );
            }
        }

        glib::ControlFlow::Continue
    });
}
