use gtk4::prelude::*;

/// Builds and registers the glassmorphic media control Popover anchored to the notch capsule.
pub fn create_media_popover(
    notch_capsule: &gtk4::Box,
) -> (
    gtk4::Popover,
    gtk4::Label,
    gtk4::Label,
    gtk4::Box,
    gtk4::Label,
    gtk4::Image,
    gtk4::Scale,    // timeline progress scale
    gtk4::Label,    // elapsed time label
    gtk4::Label,    // total time label
) {
    let popover = gtk4::Popover::new();
    popover.set_parent(notch_capsule);
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
    popover_art_container.set_valign(gtk4::Align::Fill);
    popover_art_container.set_halign(gtk4::Align::Fill);
    popover_art_container.set_hexpand(true);
    popover_art_container.set_vexpand(true);
    
    let default_popover_art = archvnde_common::icon::get_icon_colored("music", 120, "#3b82f6");
    default_popover_art.add_css_class("media-popover-art");
    default_popover_art.set_size_request(240, 240);
    default_popover_art.set_hexpand(true);
    default_popover_art.set_vexpand(true);
    default_popover_art.set_halign(gtk4::Align::Fill);
    default_popover_art.set_valign(gtk4::Align::Fill);
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

    // --- Timeline / Progress ---
    let timeline_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    timeline_box.add_css_class("media-timeline-box");
    timeline_box.set_hexpand(true);

    // Progress Scale (0.0 to 1.0)
    let progress_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 1.0, 0.001);
    progress_scale.add_css_class("media-timeline-scale");
    progress_scale.set_draw_value(false);
    progress_scale.set_hexpand(true);


    // Time labels row
    let time_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    time_row.set_hexpand(true);

    let elapsed_label = gtk4::Label::new(Some("0:00"));
    elapsed_label.add_css_class("media-time-label");
    elapsed_label.set_halign(gtk4::Align::Start);
    elapsed_label.set_hexpand(true);
    elapsed_label.set_xalign(0.0);

    let total_label = gtk4::Label::new(Some("0:00"));
    total_label.add_css_class("media-time-label");
    total_label.set_halign(gtk4::Align::End);
    total_label.set_hexpand(true);
    total_label.set_xalign(1.0);

    time_row.append(&elapsed_label);
    time_row.append(&total_label);

    timeline_box.append(&progress_scale);
    timeline_box.append(&time_row);
    popover_box.append(&timeline_box);

    // Controls
    let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 18);
    controls_box.add_css_class("media-popover-controls");
    controls_box.set_halign(gtk4::Align::Center);

    let prev_btn = gtk4::Button::new();
    prev_btn.add_css_class("media-control-btn");
    let prev_img = gtk4::Image::from_icon_name("media-skip-backward-symbolic");
    prev_img.set_pixel_size(16);
    prev_btn.set_child(Some(&prev_img));
    prev_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("playerctl").arg("previous").spawn();
    });

    let play_btn = gtk4::Button::new();
    play_btn.add_css_class("media-control-btn");
    let play_btn_icon = gtk4::Image::from_icon_name("media-playback-start-symbolic");
    play_btn_icon.set_pixel_size(22);
    play_btn.set_child(Some(&play_btn_icon));
    let play_btn_icon_clone = play_btn_icon.clone();
    play_btn.connect_clicked(move |_| {
        let _ = std::process::Command::new("playerctl").arg("play-pause").spawn();
    });

    let next_btn = gtk4::Button::new();
    next_btn.add_css_class("media-control-btn");
    let next_img = gtk4::Image::from_icon_name("media-skip-forward-symbolic");
    next_img.set_pixel_size(16);
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
    let popover_box_clone = popover_box.clone();
    
    let is_animating = std::rc::Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();

    click_gesture.connect_pressed(move |_, _, _, _| {
        if is_animating_clone.get() {
            return;
        }
        if popover_clone.is_visible() {
            let p_clone = popover_clone.clone();
            let is_animating_cb = is_animating_clone.clone();
            is_animating_cb.set(true);
            
            archvnde_common::animation::css_zoom_out_cb(
                popover_box_clone.upcast_ref(),
<<<<<<< HEAD:crates/archvnde-island/src/widgets/popover.rs
                350,
=======
                archvnde_common::animation::SlideDirection::Up,
                15,
                400,
                false,
>>>>>>> 5ea2f0f (fix: pass missing hide_after argument to slide_out_cb in popover.rs):libs/archvnde-island/src/widgets/popover.rs
                move || {
                    p_clone.popdown();
                    is_animating_cb.set(false);
                }
            );
        } else {
            popover_clone.popup();
        }
    });
    notch_capsule.add_controller(click_gesture);

    // Zoom-in when the popover maps (opens)
    let popover_box_clone2 = popover_box.clone();
    popover.connect_map(move |_| {
        archvnde_common::animation::css_zoom_in(
            popover_box_clone2.upcast_ref(),
        );
    });

    (
        popover,
        popover_title,
        popover_artist,
        popover_art_container,
        popover_app_name,
        play_btn_icon_clone,
        progress_scale,
        elapsed_label,
        total_label,
    )
}
