use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::playerctl::{load_album_art, run_playerctl};

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
        // 1. Check for incoming/active notification from D-Bus popups
        let mut active_notif = None;
        crate::notification::SHARED_NOTIFICATION.with(|sn| {
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
            let notif_icon = archvnde_common::icon::get_icon_colored(icon_symbol, 14, "#3b82f6");
            notif_icon.add_css_class("notch-album-art");
            art_container.append(&notif_icon);

            // Slide down the sub-island notification badge underneath the capsule
            if !was_custom_active.get() {
                was_custom_active.set(true);
                badge_title.set_text(&notif.title);
                badge_desc.set_text(&notif.body);

                if let Some(child) = badge_icon_container.first_child() {
                    badge_icon_container.remove(&child);
                }
                let badge_icon = archvnde_common::icon::get_icon_colored(icon_symbol, 14, "#3b82f6");
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
                    300,
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

            // 2. Fallback to playerctl music state
            let status = run_playerctl(&["status"]);
            let playing = status.as_deref() == Some("Playing");

            if playing {
                is_playing_state.set(true);

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

                play_img.set_icon_name(Some("media-playback-pause-symbolic"));

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
                        load_album_art(&art_url, 160).unwrap_or_else(|| {
                            archvnde_common::icon::get_icon_colored("music", 64, "#3b82f6")
                        })
                    } else {
                        archvnde_common::icon::get_icon_colored("music", 64, "#3b82f6")
                    };
                    new_large_art.add_css_class("media-popover-art");
                    new_large_art.set_size_request(160, 160);
                    popover_art_container.append(&new_large_art);
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
            }
        }

        glib::ControlFlow::Continue
    });
}
