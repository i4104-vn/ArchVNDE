mod playerctl;
mod visualizer;
mod player_loop;

use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use visualizer::{create_visualizer, start_visualizer_animation};
use player_loop::start_player_polling_loop;

/// Creates the macOS style dropdown notch in the panel center containing a music player.
pub fn create_system_notch() -> gtk4::Box {
    let container_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    container_vbox.set_valign(gtk4::Align::Start);
    container_vbox.set_halign(gtk4::Align::Center);

    let notch_capsule = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    notch_capsule.add_css_class("panel-notch");
    notch_capsule.set_valign(gtk4::Align::Center);
    notch_capsule.set_halign(gtk4::Align::Center);
    notch_capsule.set_visible(false);

    let notch_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    notch_content.add_css_class("notch-content");
    notch_content.set_valign(gtk4::Align::Center);
    notch_content.set_halign(gtk4::Align::Fill);
    notch_content.set_hexpand(true);

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
    music_view.set_halign(gtk4::Align::Fill);
    music_view.set_hexpand(true);
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
    track_label.set_hexpand(true);
    track_label.set_halign(gtk4::Align::Center);

    // Music Visualizer animation
    let (visualizer_box, bars) = create_visualizer();

    music_view.append(&art_container);
    music_view.append(&track_label);
    music_view.append(&visualizer_box);
    notch_content.append(&music_view);

    notch_capsule.append(&notch_content);
    container_vbox.append(&notch_capsule);

    // --- 3. Notification Badge widget ---
    let notification_badge = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    notification_badge.add_css_class("island-badge");
    notification_badge.set_valign(gtk4::Align::Start);
    notification_badge.set_halign(gtk4::Align::Center);
    notification_badge.set_visible(false); // Hidden by default

    let badge_icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    badge_icon_container.set_valign(gtk4::Align::Center);
    let badge_icon = archvnde_common::icon::get_icon_colored("bell", 14, "#3b82f6");
    badge_icon_container.append(&badge_icon);

    let badge_text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    let badge_title = gtk4::Label::new(Some("Notification"));
    badge_title.add_css_class("badge-title");
    badge_title.set_halign(gtk4::Align::Start);
    let badge_desc = gtk4::Label::new(Some("New Message"));
    badge_desc.add_css_class("badge-desc");
    badge_desc.set_halign(gtk4::Align::Start);
    
    badge_text_box.append(&badge_title);
    badge_text_box.append(&badge_desc);

    notification_badge.append(&badge_icon_container);
    notification_badge.append(&badge_text_box);
    container_vbox.append(&notification_badge);

    // --- 4. Media Control Popover ---
    let popover = gtk4::Popover::new();
    popover.set_parent(&notch_capsule);
    popover.set_has_arrow(true);
    popover.add_css_class("media-popover");

    let popover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    popover_box.add_css_class("media-popover-box");

    // Header (App Source)
    let popover_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    popover_header.add_css_class("media-popover-header");
    popover_header.set_valign(gtk4::Align::Center);
    let popover_app_icon = archvnde_common::icon::get_icon_colored("logo", 14, "#3b82f6");
    let popover_app_name = gtk4::Label::new(Some("Music Player"));
    popover_app_name.add_css_class("media-popover-app-name");
    popover_header.append(&popover_app_icon);
    popover_header.append(&popover_app_name);
    popover_box.append(&popover_header);

    // Cover Art Container
    let popover_art_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    popover_art_container.set_valign(gtk4::Align::Center);
    popover_art_container.set_halign(gtk4::Align::Center);
    
    let default_popover_art = archvnde_common::icon::get_icon_colored("music", 64, "#3b82f6");
    default_popover_art.add_css_class("media-popover-art");
    default_popover_art.set_size_request(180, 120);
    popover_art_container.append(&default_popover_art);
    popover_box.append(&popover_art_container);

    // Title & Artist
    let popover_title = gtk4::Label::new(Some("Unknown Title"));
    popover_title.add_css_class("media-popover-title");
    popover_title.set_halign(gtk4::Align::Center);
    popover_title.set_justify(gtk4::Justification::Center);
    popover_title.set_wrap(true);
    popover_title.set_max_width_chars(25);

    let popover_artist = gtk4::Label::new(Some("Unknown Artist"));
    popover_artist.add_css_class("media-popover-artist");
    popover_artist.set_halign(gtk4::Align::Center);
    popover_artist.set_justify(gtk4::Justification::Center);
    popover_artist.set_wrap(true);
    popover_artist.set_max_width_chars(30);

    popover_box.append(&popover_title);
    popover_box.append(&popover_artist);

    // Controls
    let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    controls_box.add_css_class("media-popover-controls");
    controls_box.set_halign(gtk4::Align::Center);

    let prev_btn = gtk4::Button::new();
    prev_btn.add_css_class("media-control-btn");
    let prev_img = gtk4::Image::from_icon_name("media-skip-backward-symbolic");
    prev_btn.set_child(Some(&prev_img));
    prev_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("playerctl").arg("previous").spawn();
    });

    let play_btn = gtk4::Button::new();
    play_btn.add_css_class("media-control-btn");
    let play_img = gtk4::Image::from_icon_name("media-playback-start-symbolic");
    play_btn.set_child(Some(&play_img));
    let play_img_clone = play_img.clone();
    play_btn.connect_clicked(move |_| {
        let _ = std::process::Command::new("playerctl").arg("play-pause").spawn();
    });

    let next_btn = gtk4::Button::new();
    next_btn.add_css_class("media-control-btn");
    let next_img = gtk4::Image::from_icon_name("media-skip-forward-symbolic");
    next_btn.set_child(Some(&next_img));
    next_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("playerctl").arg("next").spawn();
    });

    controls_box.append(&prev_btn);
    controls_box.append(&play_btn);
    controls_box.append(&next_btn);
    popover_box.append(&controls_box);

    popover.set_child(Some(&popover_box));

    // Toggle popover on notch_capsule click
    let click_gesture = gtk4::GestureClick::new();
    let popover_clone = popover.clone();
    click_gesture.connect_pressed(move |_, _, _, _| {
        if popover_clone.is_visible() {
            popover_clone.popdown();
        } else {
            popover_clone.popup();
        }
    });
    notch_capsule.add_controller(click_gesture);

    // Shared state variables
    let is_playing_state = Rc::new(Cell::new(false));

    // Start background animation loops
    start_visualizer_animation(bars, is_playing_state.clone());
    start_player_polling_loop(
        is_playing_state.clone(),
        notch_capsule.clone(),
        default_view,
        music_view,
        track_label,
        art_container,
        notification_badge,
        badge_title,
        badge_desc,
        badge_icon_container,
        
        // Pass popover update targets
        popover_title,
        popover_artist,
        popover_art_container,
        popover_app_name,
        play_img_clone,
    );

    container_vbox
}
