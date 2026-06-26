use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::playerctl::{load_album_art, run_playerctl};
use crate::models::IslandWidgets;

/// Starts a background timer loop that polls active D-Bus notifications and playerctl
/// state every second. It orchestrates the Dynamic Island layout updates (compact logo,
/// active notification, or media player) and updates their corresponding widgets.
pub fn start_player_polling_loop(
    is_playing_state: Rc<Cell<bool>>,
    widgets: IslandWidgets,
) {
    let last_art_url = Rc::new(RefCell::new(String::new()));
    let last_attempted_url = Rc::new(RefCell::new(String::new()));
    let fail_count = Rc::new(Cell::new(0u32));
    let was_custom_active = Rc::new(Cell::new(false));
    let poll_counter = Rc::new(Cell::new(0u32));
    let last_title = Rc::new(RefCell::new(String::new()));
    let art_loaded_for_current_song = Rc::new(Cell::new(false));

    // Create a GLib channel to send playerctl metadata from the background thread to the main thread
    let (sender, receiver) = glib::MainContext::channel::<Option<String>>(glib::Priority::default());

    // Spawn background thread to poll playerctl metadata every second
    std::thread::spawn(move || {
        loop {
            let metadata = run_playerctl(&["metadata", "--format", "{{ status }}|//|{{ title }}|//|{{ artist }}|//|{{ playerName }}|//|{{ mpris:artUrl }}"]);
            if sender.send(metadata).is_err() {
                break; // Exit thread if receiver has been dropped
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    let latest_metadata = Rc::new(RefCell::new(None));
    let latest_metadata_clone = latest_metadata.clone();

    // Hook up receiver to cache the latest metadata on the main thread
    receiver.attach(None, move |metadata| {
        *latest_metadata_clone.borrow_mut() = metadata;
        glib::ControlFlow::Continue
    });

    let (art_sender, mut art_receiver) = tokio::sync::mpsc::unbounded_channel::<(String, String, Option<(gdk_pixbuf::Pixbuf, gdk_pixbuf::Pixbuf)>)>();
    let widgets_clone = widgets.clone();
    let last_art_url_clone = last_art_url.clone();
    let last_attempted_url_clone = last_attempted_url.clone();
    let art_loaded_clone = art_loaded_for_current_song.clone();
    let fail_count_clone = fail_count.clone();

    glib::MainContext::default().spawn_local(async move {
        while let Some((url, app_icon_name, result)) = art_receiver.recv().await {
            if url == *last_attempted_url_clone.borrow() {
                if let Some((s_pb, l_pb)) = result {
                    *last_art_url_clone.borrow_mut() = url;
                    art_loaded_clone.set(true);

                    let s_texture = gdk4::Texture::for_pixbuf(&s_pb);
                    let s_art = gtk4::Image::from_paintable(Some(&s_texture));
                    s_art.set_pixel_size(16);

                    let l_texture = gdk4::Texture::for_pixbuf(&l_pb);
                    let l_art = gtk4::Image::from_paintable(Some(&l_texture));
                    l_art.set_pixel_size(240);

                    if let Some(child) = widgets_clone.art_container.first_child() {
                        widgets_clone.art_container.remove(&child);
                    }
                    s_art.add_css_class("notch-album-art");
                    widgets_clone.art_container.append(&s_art);

                    if let Some(child) = widgets_clone.popover_art_container.first_child() {
                        widgets_clone.popover_art_container.remove(&child);
                    }
                    l_art.add_css_class("media-popover-art");
                    l_art.set_size_request(240, 240);
                    l_art.set_hexpand(true);
                    l_art.set_vexpand(true);
                    l_art.set_halign(gtk4::Align::Fill);
                    l_art.set_valign(gtk4::Align::Fill);
                    widgets_clone.popover_art_container.append(&l_art);
                } else {
                    let current_fails = fail_count_clone.get() + 1;
                    fail_count_clone.set(current_fails);
                    if current_fails >= 3 {
                        *last_art_url_clone.borrow_mut() = url;
                        art_loaded_clone.set(true);

                        if let Some(child) = widgets_clone.art_container.first_child() {
                            widgets_clone.art_container.remove(&child);
                        }
                        let music_icon_s = archvnde_common::icon::get_icon_colored(&app_icon_name, 14, "#3b82f6");
                        music_icon_s.add_css_class("notch-album-art");
                        widgets_clone.art_container.append(&music_icon_s);

                        if let Some(child) = widgets_clone.popover_art_container.first_child() {
                            widgets_clone.popover_art_container.remove(&child);
                        }
                        let music_icon_l = archvnde_common::icon::get_icon_colored(&app_icon_name, 120, "#3b82f6");
                        music_icon_l.add_css_class("media-popover-art");
                        music_icon_l.set_size_request(240, 240);
                        music_icon_l.set_hexpand(true);
                        music_icon_l.set_vexpand(true);
                        music_icon_l.set_halign(gtk4::Align::Fill);
                        music_icon_l.set_valign(gtk4::Align::Fill);
                        widgets_clone.popover_art_container.append(&music_icon_l);
                    } else {
                        *last_attempted_url_clone.borrow_mut() = String::new();
                    }
                }
            }
        }
    });

    // Main thread loop to check notifications and update player view from the cached metadata
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
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
            update_notification_view(
                &widgets,
                &notif,
                &is_playing_state,
                &last_art_url,
                &last_attempted_url,
                &was_custom_active,
            );
        } else {
            if was_custom_active.get() {
                was_custom_active.set(false);
                archvnde_common::animation::slide_out(
                    widgets.notification_badge.clone().upcast_ref(),
                    archvnde_common::animation::SlideDirection::Up,
                    8,
                    200,
                    true,
                );
            }

            // Read the non-blocking cached metadata from the channel
            let metadata = latest_metadata.borrow().clone();
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
                        update_player_view(
                            &widgets,
                            &is_playing_state,
                            &poll_counter,
                            &last_title,
                            &art_loaded_for_current_song,
                            &last_art_url,
                            &last_attempted_url,
                            &fail_count,
                            status_str == "Playing",
                            &title,
                            &artist,
                            &player_name_raw,
                            &art_url,
                            &art_sender,
                        );
                    }
                }
            }

            if !player_active {
                handle_inactive_player(
                    &widgets,
                    &is_playing_state,
                    &poll_counter,
                    &last_title,
                    &art_loaded_for_current_song,
                );
            }
        }

        glib::ControlFlow::Continue
    });
}

/// Updates the Dynamic Island views to display incoming D-Bus notification details,
/// resolves system icon paths, and triggers slide-down animation for the notification badge.
fn update_notification_view(
    widgets: &IslandWidgets,
    notif: &crate::models::ActiveNotification,
    is_playing_state: &Cell<bool>,
    last_art_url: &RefCell<String>,
    last_attempted_url: &RefCell<String>,
    was_custom_active: &Cell<bool>,
) {
    is_playing_state.set(false);

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
    widgets.track_label.set_text(&display_text);

    if let Some(child) = widgets.art_container.first_child() {
        widgets.art_container.remove(&child);
    }
    let icon_symbol = if notif.icon.is_empty() { "preferences-system-notifications-symbolic" } else { &notif.icon };
    let notif_icon = archvnde_common::icon::get_system_or_file_icon(icon_symbol, "preferences-system-notifications-symbolic");
    notif_icon.set_pixel_size(14);
    notif_icon.add_css_class("notch-album-art");
    widgets.art_container.append(&notif_icon);

    *last_art_url.borrow_mut() = String::new();
    *last_attempted_url.borrow_mut() = String::new();

    if !was_custom_active.get() {
        was_custom_active.set(true);
        widgets.badge_title.set_text(&notif.title);
        widgets.badge_desc.set_text(&notif.body);

        if let Some(child) = widgets.badge_icon_container.first_child() {
            widgets.badge_icon_container.remove(&child);
        }
        let badge_icon = archvnde_common::icon::get_system_or_file_icon(icon_symbol, "preferences-system-notifications-symbolic");
        badge_icon.set_pixel_size(14);
        widgets.badge_icon_container.append(&badge_icon);

        widgets.notification_badge.set_visible(true);
        archvnde_common::animation::slide_in(
            widgets.notification_badge.clone().upcast_ref(),
            archvnde_common::animation::SlideDirection::Down,
            8,
            200,
        );
    }

    widgets.default_view.set_visible(false);
    widgets.music_view.set_visible(true);
    if !widgets.notch_capsule.is_visible() {
        widgets.notch_capsule.add_css_class("active-music");
        archvnde_common::animation::island_zoom_in(
            widgets.notch_capsule.clone().upcast_ref(),
            200,
            10,
            500,
        );
    }
}

fn get_player_icon_name(player_name_raw: &str) -> String {
    let lower_player = player_name_raw.to_lowercase();
    if lower_player.is_empty() {
        return "music".to_string();
    }

    // Dynamic search across registered desktop application entry files
    let apps = archvnde_common::find_desktop_apps();
    for app in &apps {
        let app_name = app.name.to_lowercase();
        let app_exec = app.exec.to_lowercase();
        if app_exec.contains(&lower_player) || app_name.contains(&lower_player) {
            if let Some(ref icon) = app.icon {
                return icon.clone();
            }
        }
    }

    // Direct fallback using the raw name
    lower_player
}

/// Updates the Dynamic Island views to display metadata from the active media player,
/// handles loading and scaling cover artwork with failure retries, and synchronizes popover controls.
fn update_player_view(
    widgets: &IslandWidgets,
    is_playing_state: &Cell<bool>,
    poll_counter: &Cell<u32>,
    last_title: &RefCell<String>,
    art_loaded_for_current_song: &Cell<bool>,
    last_art_url: &RefCell<String>,
    last_attempted_url: &RefCell<String>,
    fail_count: &Cell<u32>,
    playing: bool,
    title: &str,
    artist: &str,
    player_name_raw: &str,
    art_url: &str,
    art_sender: &tokio::sync::mpsc::UnboundedSender<(String, String, Option<(gdk_pixbuf::Pixbuf, gdk_pixbuf::Pixbuf)>)>,
) {
    is_playing_state.set(playing);

    let song_changed = {
        let mut last_title_borrow = last_title.borrow_mut();
        if title != *last_title_borrow {
            *last_title_borrow = title.to_string();
            true
        } else {
            false
        }
    };

    if song_changed {
        art_loaded_for_current_song.set(false);
        poll_counter.set(0);
    }

    let count = poll_counter.get();
    poll_counter.set(count + 1);

    if song_changed || count % 5 == 0 {
        let label_text = if artist.is_empty() {
            title.to_string()
        } else {
            format!("{} - {}", artist, title)
        };

        let display_text = if label_text.chars().count() > 18 {
            let truncated: String = label_text.chars().take(15).collect();
            format!("{}...", truncated)
        } else {
            label_text
        };
        widgets.track_label.set_text(&display_text);

        widgets.popover_title.set_text(title);
        widgets.popover_artist.set_text(artist);

        let player_name = if !player_name_raw.is_empty() {
            let mut chars = player_name_raw.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        } else {
            "Music Player".to_string()
        };
        widgets.popover_app_name.set_text(&player_name);
    }

    if !art_loaded_for_current_song.get() {
        let app_icon_name = get_player_icon_name(player_name_raw);
        let mut last_url = last_art_url.borrow_mut();

        if art_url.is_empty() {
            *last_url = art_url.to_string();
            art_loaded_for_current_song.set(true);

            if let Some(child) = widgets.art_container.first_child() {
                widgets.art_container.remove(&child);
            }
            let music_icon_s = archvnde_common::icon::get_icon_colored(&app_icon_name, 14, "#3b82f6");
            music_icon_s.add_css_class("notch-album-art");
            widgets.art_container.append(&music_icon_s);

            if let Some(child) = widgets.popover_art_container.first_child() {
                widgets.popover_art_container.remove(&child);
            }
            let music_icon_l = archvnde_common::icon::get_icon_colored(&app_icon_name, 120, "#3b82f6");
            music_icon_l.add_css_class("media-popover-art");
            music_icon_l.set_size_request(240, 240);
            music_icon_l.set_hexpand(true);
            music_icon_l.set_vexpand(true);
            music_icon_l.set_halign(gtk4::Align::Fill);
            music_icon_l.set_valign(gtk4::Align::Fill);
            widgets.popover_art_container.append(&music_icon_l);
        } else {
            if art_url != *last_attempted_url.borrow() {
                *last_attempted_url.borrow_mut() = art_url.to_string();
                fail_count.set(0);

                let art_url_clone = art_url.to_string();
                let app_icon_name_clone = app_icon_name.clone();
                let art_sender_clone = art_sender.clone();

                std::thread::spawn(move || {
                    let small_pb = super::playerctl::load_album_art_pixbuf(&art_url_clone, 16);
                    let large_pb = super::playerctl::load_album_art_pixbuf(&art_url_clone, 240);
                    if let (Some(s_pb), Some(l_pb)) = (small_pb, large_pb) {
                        let _ = art_sender_clone.send((art_url_clone, app_icon_name_clone, Some((s_pb, l_pb))));
                    } else {
                        let _ = art_sender_clone.send((art_url_clone, app_icon_name_clone, None));
                    }
                });
            }
        }
    }

    if playing {
        widgets.play_btn_icon.set_icon_name(Some("media-playback-pause-symbolic"));
    } else {
        widgets.play_btn_icon.set_icon_name(Some("media-playback-start-symbolic"));
    }

    widgets.default_view.set_visible(false);
    widgets.music_view.set_visible(true);
    if !widgets.notch_capsule.is_visible() {
        widgets.notch_capsule.add_css_class("active-music");
        archvnde_common::animation::island_zoom_in(
            widgets.notch_capsule.clone().upcast_ref(),
            200,
            10,
            500,
        );
    }
}

/// Cleans up visual elements (resets metadata, clears images) and triggers exit zoom-out
/// animations when the media player is no longer active.
fn handle_inactive_player(
    widgets: &IslandWidgets,
    is_playing_state: &Cell<bool>,
    poll_counter: &Cell<u32>,
    last_title: &RefCell<String>,
    art_loaded_for_current_song: &Cell<bool>,
) {
    is_playing_state.set(false);
    poll_counter.set(0);
    last_title.borrow_mut().clear();
    art_loaded_for_current_song.set(false);

    widgets.play_btn_icon.set_icon_name(Some("media-playback-start-symbolic"));

    if let Some(child) = widgets.art_container.first_child() {
        widgets.art_container.remove(&child);
    }
    if let Some(child) = widgets.popover_art_container.first_child() {
        widgets.popover_art_container.remove(&child);
    }

    if widgets.notch_capsule.is_visible() {
        let notch_capsule_clone = widgets.notch_capsule.clone();
        archvnde_common::animation::island_zoom_out(
            widgets.notch_capsule.clone().upcast_ref(),
            200,
            500,
            true,
        );
        glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
            notch_capsule_clone.remove_css_class("active-music");
        });
    }
}
