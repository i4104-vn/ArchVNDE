mod playerctl;
mod player_loop;
pub mod widgets;

use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use widgets::visualizer::{create_visualizer, start_visualizer_animation};
use widgets::popover;
use widgets::notification;
use player_loop::start_player_polling_loop;

/// Creates the macOS style dropdown island in the panel center containing a music player.
pub fn create_system_island() -> gtk4::Box {
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

    // --- 3. Notification Badge container (vertical stack, holds up to 3 active badges) ---
    let notification_badge = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    notification_badge.set_valign(gtk4::Align::Start);
    notification_badge.set_halign(gtk4::Align::Center);
    container_vbox.append(&notification_badge);

    // Dummy elements to satisfy start_player_polling_loop arguments
    let badge_title = gtk4::Label::new(None);
    let badge_desc = gtk4::Label::new(None);
    let badge_icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    // --- 4. Media Control Popover ---
    let (
        _popover,
        popover_title,
        popover_artist,
        popover_art_container,
        popover_app_name,
        play_btn_icon,
    ) = widgets::popover::create_media_popover(&notch_capsule);

    // Shared state variables
    let is_playing_state = Rc::new(Cell::new(false));

    // Spawn DBus listener on startup
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<widgets::notification::NotificationMsg>();
    widgets::notification::spawn_dbus_listener(tx);

    let notif_badge_clone = notification_badge.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                widgets::notification::NotificationMsg::New { summary, body, icon, timeout } => {
                    widgets::notification::show_notification_popup(&summary, &body, &icon, timeout);

                    // Create dynamic badge card
                    let badge_card = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                    badge_card.add_css_class("island-badge");
                    badge_card.set_valign(gtk4::Align::Center);

                    let badge_icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
                    badge_icon_box.set_valign(gtk4::Align::Center);
                    let icon_symbol = if icon.is_empty() { "bell" } else { &icon };
                    let badge_icon = archvnde_common::icon::get_icon_colored(icon_symbol, 14, "#3b82f6");
                    badge_icon_box.append(&badge_icon);

                    let badge_text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
                    let badge_lbl_title = gtk4::Label::new(Some(&summary));
                    badge_lbl_title.add_css_class("badge-title");
                    badge_lbl_title.set_halign(gtk4::Align::Start);
                    
                    let badge_lbl_desc = gtk4::Label::new(Some(&body));
                    badge_lbl_desc.add_css_class("badge-desc");
                    badge_lbl_desc.set_halign(gtk4::Align::Start);
                    badge_lbl_desc.set_wrap(true);
                    badge_lbl_desc.set_max_width_chars(28);

                    badge_text_box.append(&badge_lbl_title);
                    badge_text_box.append(&badge_lbl_desc);

                    badge_card.append(&badge_icon_box);
                    badge_card.append(&badge_text_box);

                    // Prepend and animate in
                    notif_badge_clone.prepend(&badge_card);
                    archvnde_common::animation::fade_in(
                        badge_card.upcast_ref(),
                        250,
                    );

                    // Keep maximum of 3 active badges
                    let mut children = Vec::new();
                    let mut next_child = notif_badge_clone.first_child();
                    while let Some(child) = next_child {
                        next_child = child.next_sibling();
                        children.push(child);
                    }

                    if children.len() > 3 {
                        let items_to_remove = children.len() - 3;
                        for i in 0..items_to_remove {
                            let old_idx = children.len() - 1 - i;
                            let old_card = children[old_idx].clone();
                            let nb_c = notif_badge_clone.clone();
                            let old_card_clone = old_card.clone();
                            archvnde_common::animation::fade_out_cb(
                                &old_card,
                                250,
                                move || {
                                    nb_c.remove(&old_card_clone);
                                }
                            );
                        }
                    }

                    // Expire card after 5 seconds
                    let badge_card_expire = badge_card.clone();
                    let nb_expire = notif_badge_clone.clone();
                    glib::timeout_add_local_once(std::time::Duration::from_secs(5), move || {
                        if badge_card_expire.parent().is_some() {
                            let bce_clone = badge_card_expire.clone();
                            archvnde_common::animation::fade_out_cb(
                                badge_card_expire.upcast_ref(),
                                250,
                                move || {
                                    nb_expire.remove(&bce_clone);
                                }
                            );
                        }
                    });
                }
                widgets::notification::NotificationMsg::Close => {
                    widgets::notification::close_notification_popup();
                }
            }
        }
    });

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
        play_btn_icon,
    );

    container_vbox
}
