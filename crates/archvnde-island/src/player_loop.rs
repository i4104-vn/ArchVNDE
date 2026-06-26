use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::playerctl::{load_album_art, run_playerctl};

/// Formats seconds into M:SS string
fn format_time(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

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

    // Timeline widgets
    progress_scale: gtk4::Scale,
    elapsed_label: gtk4::Label,
    total_label: gtk4::Label,
) {
    let last_art_url = Rc::new(RefCell::new(String::new()));
    let last_attempted_url = Rc::new(RefCell::new(String::new()));
    let fail_count = Rc::new(Cell::new(0u32));
    let was_custom_active = Rc::new(Cell::new(false));

    // Track user seeking with a timestamp to avoid overwriting seek values and slider jumping
    let last_seek_time = Rc::new(Cell::new(std::time::Instant::now()));

    let last_seek_time_clone = last_seek_time.clone();
    progress_scale.connect_change_value(move |_scale, _scroll_type, value| {
        last_seek_time_clone.set(std::time::Instant::now());
        // Get duration in microseconds from playerctl
        if let Some(length_str) = crate::playerctl::run_playerctl(&["metadata", "mpris:length"]) {
            if let Ok(length_us) = length_str.parse::<f64>() {
                let seek_pos = value * length_us;
                let _ = std::process::Command::new("playerctl")
                    .arg("position")
                    .arg(format!("{:.6}", seek_pos / 1_000_000.0))
                    .spawn();
            }
        }
        glib::Propagation::Proceed
    });

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
            let icon_symbol = if notif.icon.is_empty() { "bell" } else { &notif.icon };
            let notif_icon = if icon_symbol.starts_with('/') {
                gtk4::Image::from_file(icon_symbol)
            } else {
                let name = if icon_symbol == "bell" {
                    "preferences-system-notifications-symbolic"
                } else {
                    icon_symbol
                };
                gtk4::Image::from_icon_name(name)
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
                    let name = if icon_symbol == "message" {
                        "preferences-system-notifications-symbolic"
                    } else {
                        icon_symbol
                    };
                    gtk4::Image::from_icon_name(name)
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
                archvnde_common::animation::zoom_in(
                    notch_capsule.clone().upcast_ref(),
                    200,
                    22,
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
            let status = run_playerctl(&["status"]);
            let status_str = status.as_deref().unwrap_or("None");

            if status_str == "Playing" || status_str == "Paused" {
                let playing = status_str == "Playing";
                is_playing_state.set(playing);

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

                if playing {
                    play_img.set_icon_name(Some("media-playback-pause-symbolic"));
                } else {
                    play_img.set_icon_name(Some("media-playback-start-symbolic"));
                }

                // --- Update timeline ---
                let length_us = run_playerctl(&["metadata", "mpris:length"])
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let position_secs = run_playerctl(&["position"])
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let duration_secs = length_us / 1_000_000.0;

                elapsed_label.set_text(&format_time(position_secs));
                total_label.set_text(&format_time(duration_secs));

                // Only update scale from current player time if user hasn't seeking recently (gives player time to seek)
                if last_seek_time.get().elapsed() > std::time::Duration::from_secs(2) && duration_secs > 0.0 {
                    let fraction = (position_secs / duration_secs).clamp(0.0, 1.0);
                    progress_scale.set_value(fraction);
                }

                // Check art url — only update cached URL on successful art load (or empty)
                let art_url = run_playerctl(&["metadata", "mpris:artUrl"]).unwrap_or_default();
                let mut last_url = last_art_url.borrow_mut();
                let mut last_attempt = last_attempted_url.borrow_mut();

                if *last_attempt != art_url {
                    *last_attempt = art_url.clone();
                    fail_count.set(0);
                }

                if *last_url != art_url {
                    if art_url.is_empty() {
                        *last_url = art_url.clone();
                        
                        // Set fallback icon
                        if let Some(child) = art_container.first_child() {
                            art_container.remove(&child);
                        }
                        let new_art = archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6");
                        new_art.add_css_class("notch-album-art");
                        art_container.append(&new_art);

                        if let Some(child) = popover_art_container.first_child() {
                            popover_art_container.remove(&child);
                        }
                        let new_large_art = archvnde_common::icon::get_icon_colored("music", 120, "#3b82f6");
                        new_large_art.add_css_class("media-popover-art");
                        new_large_art.set_size_request(240, 240);
                        new_large_art.set_hexpand(true);
                        new_large_art.set_vexpand(true);
                        new_large_art.set_halign(gtk4::Align::Fill);
                        new_large_art.set_valign(gtk4::Align::Fill);
                        popover_art_container.append(&new_large_art);
                    } else {
                        let small_art = load_album_art(&art_url, 16);
                        let large_art = load_album_art(&art_url, 240);

                        if let (Some(s_art), Some(l_art)) = (small_art, large_art) {
                            *last_url = art_url.clone();

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

                                if let Some(child) = art_container.first_child() {
                                    art_container.remove(&child);
                                }
                                let new_art = archvnde_common::icon::get_icon_colored("music", 14, "#3b82f6");
                                new_art.add_css_class("notch-album-art");
                                art_container.append(&new_art);

                                if let Some(child) = popover_art_container.first_child() {
                                    popover_art_container.remove(&child);
                                }
                                let new_large_art = archvnde_common::icon::get_icon_colored("music", 120, "#3b82f6");
                                new_large_art.add_css_class("media-popover-art");
                                new_large_art.set_size_request(240, 240);
                                new_large_art.set_hexpand(true);
                                new_large_art.set_vexpand(true);
                                new_large_art.set_halign(gtk4::Align::Fill);
                                new_large_art.set_valign(gtk4::Align::Fill);
                                popover_art_container.append(&new_large_art);
                            }
                        }
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
                        500,
                    );
                }
            } else {
                is_playing_state.set(false);

                // Reset timeline when not playing
                progress_scale.set_value(0.0);
                elapsed_label.set_text("0:00");
                total_label.set_text("0:00");
                play_img.set_icon_name(Some("media-playback-start-symbolic"));

                if notch_capsule.is_visible() {
                    let notch_capsule_clone = notch_capsule.clone();
                    archvnde_common::animation::zoom_out(
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
