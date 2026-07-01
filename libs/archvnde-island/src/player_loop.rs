use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::playerctl::{load_album_art, run_playerctl};

/// Formats seconds into M:SS string
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
    play_btn_icon: gtk4::Image,
) {
    let last_art_url = Rc::new(RefCell::new(String::new()));
    let last_attempted_url = Rc::new(RefCell::new(String::new()));
    let fail_count = Rc::new(Cell::new(0u32));
    let was_custom_active = Rc::new(Cell::new(false));

    // Poll counter state
    let poll_counter = Rc::new(Cell::new(0u32));

    let last_title = Rc::new(RefCell::new(String::new()));
    let art_loaded_for_current_song = Rc::new(Cell::new(false));

    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        // 1. Check for incoming/active notification from D-Bus popups
        let mut active_notif = None;
        crate::widgets::notification::SHARED_NOTIFICATION.with(|sn| {
            if let Some(ref notif) = *sn.borrow() {
                if notif.timestamp.elapsed() < std::time::Duration::from_secs(5) {
                    active_notif = Some(notif.clone());
                }
            }
            if active_notif.is_none() {
                *sn.borrow_mut() = None;
            }
        });

        if let Some(notif) = active_notif {
            // Disable music visualizer while showing notification
            is_playing_state.set(false);

            // Update track label with notification details
            let display_text = if notif.body.is_empty() {
                notif.title.clone()
            } else {
                format!("{} - {}", notif.title, notif.body)
            };
            let display_text = if display_text.chars().count() > 18 {
                let truncated: String = display_text.chars().take(15).collect();
                format!("{}...", truncated)
            } else {
                display_text
            };
            track_label.set_text(&display_text);

            // Set notification icon in the capsule
            if let Some(child) = art_container.first_child() {
                art_container.remove(&child);
            }
            let icon_symbol = if notif.icon.is_empty() { "preferences-system-notifications-symbolic" } else { &notif.icon };
            let notif_icon = if icon_symbol.starts_with('/') {
                gtk4::Image::from_file(icon_symbol)
            } else {
                gtk4::Image::from_icon_name(icon_symbol)
            };
            notif_icon.set_pixel_size(14);
            notif_icon.add_css_class("notch-album-art");
            art_container.append(&notif_icon);

            // Clear cached art URL so album art is restored when notification expires
            *last_art_url.borrow_mut() = String::new();
            *last_attempted_url.borrow_mut() = String::new();

            // Slide down the sub-island notification badge underneath the capsule
            if !was_custom_active.get() {
                was_custom_active.set(true);
                badge_title.set_text(&notif.title);
                badge_desc.set_text(&notif.body);

                if let Some(child) = badge_icon_container.first_child() {
                    badge_icon_container.remove(&child);
                }
                let badge_icon = if icon_symbol.starts_with('/') {
                    gtk4::Image::from_file(icon_symbol)
                } else {
                    gtk4::Image::from_icon_name(icon_symbol)
                };
                badge_icon.set_pixel_size(14);
                badge_icon_container.append(&badge_icon);

                notification_badge.set_visible(true);
                archvnde_common::animation::slide_in(
                    notification_badge.clone().upcast_ref(),
                    archvnde_common::animation::SlideDirection::Down,
                    8,
                    200,
                );
            }

            default_view.set_visible(false);
            music_view.set_visible(true);
            if !notch_capsule.is_visible() {
                notch_capsule.add_css_class("active-music");
                archvnde_common::animation::island_zoom_in(
                    notch_capsule.clone().upcast_ref(),
                    200,
                    10,
                    500,
                );
            }
        } else {
            // Remove notification badge if it just expired
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

            // 2. Fallback to playerctl music state (Playing or Paused)
            let metadata = run_playerctl(&["metadata", "--format", "{{ status }}|//|{{ title }}|//|{{ artist }}|//|{{ playerName }}|//|{{ mpris:artUrl }}"]);
            let mut player_active = false;

            if let Some(ref line) = metadata {
                let parts: Vec<&str> = line.split("|//|").collect();
                if parts.len() >= 5 {
                    let status_str = parts[0].trim();
                    let title = parts[1].trim().to_string();
                    let artist = parts[2].trim().to_string();
                    let player_name_raw = parts[3].trim().to_string();
                    let art_url = parts[4].trim().to_string();

                    if status_str == "Playing" || status_str == "Paused" {
                        player_active = true;
                        let playing = status_str == "Playing";
                        is_playing_state.set(playing);

                        let song_changed = {
                            let mut last_title_borrow = last_title.borrow_mut();
                            if title != *last_title_borrow {
                                *last_title_borrow = title.clone();
                                true
                            } else {
                                false
                            }
                        };

                        if song_changed {
                            art_loaded_for_current_song.set(false);
                            poll_counter.set(0); // Force metadata update immediately
                        }

                        let count = poll_counter.get();
                        poll_counter.set(count + 1);

                        // Update metadata every 5 seconds (or immediately on song change)
                        if song_changed || count % 5 == 0 {
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

                            let player_name = if !player_name_raw.is_empty() {
                                let mut chars = player_name_raw.chars();
                                match chars.next() {
                                    None => String::new(),
                                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                                }
                            } else {
                                "Music Player".to_string()
                            };
                            popover_app_name.set_text(&player_name);
                        }

                        // Update cover art (only load when not yet loaded for the current song)
                        if !art_loaded_for_current_song.get() {
                            let mut last_url = last_art_url.borrow_mut();

                            if art_url.is_empty() {
                                *last_url = art_url.clone();
                                art_loaded_for_current_song.set(true);

                                // Clear and set fallback music icon
                                if let Some(child) = art_container.first_child() {
                                    art_container.remove(&child);
                                }
                                let music_icon_s = archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6");
                                music_icon_s.add_css_class("notch-album-art");
                                art_container.append(&music_icon_s);

                                if let Some(child) = popover_art_container.first_child() {
                                    popover_art_container.remove(&child);
                                }
                                let music_icon_l = archvnde_common::icon::get_icon_colored("music", 120, "#3b82f6");
                                music_icon_l.add_css_class("media-popover-art");
                                music_icon_l.set_size_request(240, 240);
                                music_icon_l.set_hexpand(true);
                                music_icon_l.set_vexpand(true);
                                music_icon_l.set_halign(gtk4::Align::Fill);
                                music_icon_l.set_valign(gtk4::Align::Fill);
                                popover_art_container.append(&music_icon_l);
                            } else {
                                let small_art = load_album_art(&art_url, 16);
                                let large_art = load_album_art(&art_url, 240);

                                if let (Some(s_art), Some(l_art)) = (small_art, large_art) {
                                    *last_url = art_url.clone();
                                    art_loaded_for_current_song.set(true);

                                    if let Some(child) = art_container.first_child() {
                                        art_container.remove(&child);
                                    }
                                    s_art.add_css_class("notch-album-art");
                                    art_container.append(&s_art);

                                    if let Some(child) = popover_art_container.first_child() {
                                        popover_art_container.remove(&child);
                                    }
                                    l_art.add_css_class("media-popover-art");
                                    l_art.set_size_request(240, 240);
                                    l_art.set_hexpand(true);
                                    l_art.set_vexpand(true);
                                    l_art.set_halign(gtk4::Align::Fill);
                                    l_art.set_valign(gtk4::Align::Fill);
                                    popover_art_container.append(&l_art);
                                } else {
                                    let current_fails = fail_count.get() + 1;
                                    fail_count.set(current_fails);
                                    if current_fails >= 3 {
                                        *last_url = art_url.clone();
                                        art_loaded_for_current_song.set(true); // Stop trying and use fallback icon

                                        if let Some(child) = art_container.first_child() {
                                            art_container.remove(&child);
                                        }
                                        let music_icon_s = archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6");
                                        music_icon_s.add_css_class("notch-album-art");
                                        art_container.append(&music_icon_s);

                                        if let Some(child) = popover_art_container.first_child() {
                                            popover_art_container.remove(&child);
                                        }
                                        let music_icon_l = archvnde_common::icon::get_icon_colored("music", 120, "#3b82f6");
                                        music_icon_l.add_css_class("media-popover-art");
                                        music_icon_l.set_size_request(240, 240);
                                        music_icon_l.set_hexpand(true);
                                        music_icon_l.set_vexpand(true);
                                        music_icon_l.set_halign(gtk4::Align::Fill);
                                        music_icon_l.set_valign(gtk4::Align::Fill);
                                        popover_art_container.append(&music_icon_l);
                                    }
                                }
                            }
                        }

                        // Update play/pause icon state
                        if playing {
                            play_btn_icon.set_icon_name(Some("media-playback-pause-symbolic"));
                        } else {
                            play_btn_icon.set_icon_name(Some("media-playback-start-symbolic"));
                        }

                        default_view.set_visible(false);
                        music_view.set_visible(true);
                        if !notch_capsule.is_visible() {
                            notch_capsule.add_css_class("active-music");
                            archvnde_common::animation::island_zoom_in(
                                notch_capsule.clone().upcast_ref(),
                                200,
                                10,
                                500,
                            );
                        }
                    }
                }
            }

            if !player_active {
                is_playing_state.set(false);
                poll_counter.set(0); // Reset counter
                last_title.borrow_mut().clear();
                art_loaded_for_current_song.set(false);

                play_btn_icon.set_icon_name(Some("media-playback-start-symbolic"));

                // Clear artwork (no fallback icon)
                if let Some(child) = art_container.first_child() {
                    art_container.remove(&child);
                }
                if let Some(child) = popover_art_container.first_child() {
                    popover_art_container.remove(&child);
                }

                if notch_capsule.is_visible() {
                    let notch_capsule_clone = notch_capsule.clone();
                    archvnde_common::animation::island_zoom_out(
                        notch_capsule.clone().upcast_ref(),
                        200,
                        500,
                        true,
                    );
                    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                        notch_capsule_clone.remove_css_class("active-music");
                    });
                }
            }
        }

        glib::ControlFlow::Continue
    });
}
