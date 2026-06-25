use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Creates the macOS style dropdown notch in the panel center containing a music player.
pub fn create_system_notch() -> gtk4::Box {
    let notch_capsule = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    notch_capsule.add_css_class("panel-notch");
    notch_capsule.set_valign(gtk4::Align::Start);
    notch_capsule.set_halign(gtk4::Align::Center);

    // Notch content box (so we can transition opacity of contents)
    let notch_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    notch_content.add_css_class("notch-content");
    notch_content.set_valign(gtk4::Align::Center);
    notch_content.set_halign(gtk4::Align::Center);

    // --- Music Player section ---
    let player_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    player_box.set_valign(gtk4::Align::Center);
    
    let play_btn = gtk4::Button::new();
    play_btn.add_css_class("notch-btn");
    let play_icon = archvnde_common::icon::get_icon_colored("music", 12, "#3b82f6");
    play_btn.set_child(Some(&play_icon));
    
    let is_playing = Rc::new(Cell::new(true));
    let is_playing_clone = is_playing.clone();
    let play_btn_clone = play_btn.clone();
    play_btn.connect_clicked(move |_| {
        let playing = !is_playing_clone.get();
        is_playing_clone.set(playing);
        if playing {
            play_btn_clone.set_child(Some(&archvnde_common::icon::get_icon_colored("music", 12, "#3b82f6")));
        } else {
            play_btn_clone.set_child(Some(&archvnde_common::icon::get_icon_colored("music", 12, "#94a3b8")));
        }
    });

    let track_label = gtk4::Label::new(Some("Track 01"));
    track_label.add_css_class("notch-player-text");

    player_box.append(&play_btn);
    player_box.append(&track_label);

    notch_content.append(&player_box);
    notch_capsule.append(&notch_content);

    notch_capsule
}
