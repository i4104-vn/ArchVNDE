pub mod toggle_grid;
pub mod sliders;
pub mod power_actions;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use toggle_grid::create_control_center_grid;
use sliders::create_slider_row;
use power_actions::create_header_row;

/// Creates a unified status indicators capsule containing (1) status details button and (2) clock button.
/// Clicking the status button toggles Quick Settings; clicking the clock button toggles Calendar.
/// The two panels are mutually exclusive.
pub fn create_status_indicators(
    app: &gtk4::Application,
    quick_settings_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::Box {
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    status_box.add_css_class("status-indicators-box");
    status_box.set_valign(gtk4::Align::Center);

    let status_button = gtk4::Button::new();
    status_button.add_css_class("panel-status-btn");

    let status_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);

    let bluetooth_icon = archvnde_common::icon::get_icon("bluetooth", 14);
    bluetooth_icon.add_css_class("status-icon");

    let wifi_icon = archvnde_common::icon::get_icon("wifi", 14);
    wifi_icon.add_css_class("status-icon");

    let battery_icon = archvnde_common::icon::get_icon("battery", 14);
    battery_icon.add_css_class("status-icon");
    let battery_percent = gtk4::Label::new(Some("100%"));
    battery_percent.add_css_class("status-text");

    status_content.append(&wifi_icon);
    status_content.append(&bluetooth_icon);
    status_content.append(&battery_icon);
    status_content.append(&battery_percent);

    status_button.set_child(Some(&status_content));

    let qsw_clone = quick_settings_window.clone();
    let cw_clone = calendar_window.clone();
    let app_clone = app.clone();
    status_button.connect_clicked(move |_| {
        let cal_win = {
            cw_clone.borrow().clone()
        };
        if let Some(win) = cal_win {
            win.close();
        }

        let existing = {
            let borrow = qsw_clone.borrow();
            borrow.clone()
        };
        if let Some(existing_window) = existing {
            existing_window.close();
        } else {
            let q_win = create_quick_settings_window(&app_clone, qsw_clone.clone());
            if let Ok(mut borrow) = qsw_clone.try_borrow_mut() {
                *borrow = Some(q_win);
            }
        }
    });

    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");

    let clock_button = crate::widgets::clock::create_clock_widget(
        app,
        quick_settings_window.clone(),
        calendar_window.clone(),
    );

    status_box.append(&status_button);
    status_box.append(&separator);
    status_box.append(&clock_button);

    status_box
}

/// Builds and maps a glassmorphic Quick Settings popup ApplicationWindow anchored
/// to the top-right corner. It binds volume and brightness sliders, grid settings toggles,
/// and registers Genie animations on close and map events.
fn create_quick_settings_window(
    app: &gtk4::Application,
    quick_settings_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::ApplicationWindow {
    use gtk4_layer_shell::{KeyboardMode, Layer, Edge};

    let q_win = gtk4::ApplicationWindow::new(app);
    q_win.init_layer_shell();
    q_win.set_layer(Layer::Overlay);
    q_win.set_keyboard_mode(KeyboardMode::OnDemand);

    q_win.set_anchor(Edge::Top, true);
    q_win.set_anchor(Edge::Right, true);
    q_win.set_margin(Edge::Top, 10);
    q_win.set_margin(Edge::Right, 12);
    q_win.set_default_size(360, 480);
    q_win.add_css_class("quick-settings-window");

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    main_box.add_css_class("quick-settings-box");
    main_box.set_valign(gtk4::Align::Start);

    main_box.append(&create_header_row());
    main_box.append(&create_control_center_grid());

    main_box.append(&create_slider_row("volume", 80.0, |val| {
        println!("Volume changed: {}%", val as i32);
    }));

    main_box.append(&create_slider_row("brightness", 60.0, |val| {
        println!("Brightness changed: {}%", val as i32);
    }));

    let (media_box, media_widgets) = create_media_player_box();
    main_box.append(&media_box);

    q_win.set_child(Some(&main_box));

    q_win.connect_is_active_notify(|win| {
        if !win.is_active() {
            win.close();
        }
    });

    let is_animating = Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();
    let qsw_inner = quick_settings_window.clone();
    let q_win_clone = q_win.clone();
    let main_box_clone = main_box.clone();
    q_win.connect_close_request(move |_| {
        if is_animating_clone.get() {
            return glib::Propagation::Proceed;
        }
        is_animating_clone.set(true);
        if let Ok(mut borrow) = qsw_inner.try_borrow_mut() {
            *borrow = None;
        }
        let q_win_cb = q_win_clone.clone();
        archvnde_common::animation::genie_out(
            main_box_clone.upcast_ref(),
            360,
            480,
            400,
            move || {
                q_win_cb.destroy();
            }
        );
        glib::Propagation::Stop
    });

    q_win.present();
    archvnde_common::animation::genie_in(main_box.upcast_ref(), 360, 480, 400);

    start_control_player_loop(media_widgets, &q_win);

    q_win
}

struct ControlMediaWidgets {
    art_image: gtk4::Image,
    title_label: gtk4::Label,
    artist_label: gtk4::Label,
    play_icon: gtk4::Image,
}

fn run_playerctl(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("playerctl")
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn load_album_art(url: &str, size: i32) -> Option<gtk4::Image> {
    if url.is_empty() {
        return None;
    }
    let path = if url.starts_with("file://") {
        url.trim_start_matches("file://").to_string()
    } else if url.starts_with('/') {
        url.to_string()
    } else {
        return None;
    };

    if std::path::Path::new(&path).exists() {
        let pb = gdk_pixbuf::Pixbuf::from_file_at_scale(&path, size, size, true).ok()?;
        let texture = gdk4::Texture::for_pixbuf(&pb);
        let img = gtk4::Image::from_paintable(Some(&texture));
        img.set_pixel_size(size);
        Some(img)
    } else {
        None
    }
}

fn get_player_icon_name(player_name_raw: &str) -> String {
    let lower_player = player_name_raw.to_lowercase();
    if lower_player.is_empty() {
        return "music".to_string();
    }

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

    lower_player
}

fn create_media_player_box() -> (gtk4::Box, std::rc::Rc<ControlMediaWidgets>) {
    let media_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    media_box.add_css_class("control-media-box");

    let art_image = gtk4::Image::new();
    art_image.add_css_class("control-music-art");
    art_image.set_size_request(48, 48);

    let default_icon = archvnde_common::icon::get_icon_colored("music", 20, "#3b82f6");
    art_image.set_paintable(default_icon.paintable().as_ref());

    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_valign(gtk4::Align::Center);

    let title_label = gtk4::Label::new(Some("No media"));
    title_label.add_css_class("control-music-title");
    title_label.set_xalign(0.0);
    title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    let artist_label = gtk4::Label::new(Some("Unknown Artist"));
    artist_label.add_css_class("control-music-artist");
    artist_label.set_xalign(0.0);
    artist_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    text_box.append(&title_label);
    text_box.append(&artist_label);

    let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    controls_box.set_valign(gtk4::Align::Center);

    let play_btn = gtk4::Button::new();
    play_btn.add_css_class("control-music-btn");
    let play_icon = gtk4::Image::from_icon_name("media-playback-start-symbolic");
    play_icon.set_pixel_size(16);
    play_btn.set_child(Some(&play_icon));

    play_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("playerctl").arg("play-pause").spawn();
    });

    let next_btn = gtk4::Button::new();
    next_btn.add_css_class("control-music-btn");
    let next_icon = gtk4::Image::from_icon_name("media-skip-forward-symbolic");
    next_icon.set_pixel_size(16);
    next_btn.set_child(Some(&next_icon));

    next_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("playerctl").arg("next").spawn();
    });

    controls_box.append(&play_btn);
    controls_box.append(&next_btn);

    media_box.append(&art_image);
    media_box.append(&text_box);
    media_box.append(&controls_box);

    let widgets = std::rc::Rc::new(ControlMediaWidgets {
        art_image,
        title_label,
        artist_label,
        play_icon,
    });

    (media_box, widgets)
}

fn start_control_player_loop(widgets: std::rc::Rc<ControlMediaWidgets>, win: &gtk4::ApplicationWindow) {
    let win_weak = win.downgrade();
    
    let last_title = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let last_art_url = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let last_play_state = std::rc::Rc::new(std::cell::Cell::new(false));

    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let win = match win_weak.upgrade() {
            Some(w) => w,
            None => return gtk4::glib::ControlFlow::Break,
        };

        if !win.is_active() {
            return gtk4::glib::ControlFlow::Break;
        }

        let metadata = run_playerctl(&[
            "metadata",
            "--format",
            "{{ status }}|//|{{ title }}|//|{{ artist }}|//|{{ playerName }}|//|{{ mpris:artUrl }}",
        ]);

        if let Some(meta_str) = metadata {
            let parts: Vec<&str> = meta_str.split("|//|").collect();
            if parts.len() >= 5 {
                let status = parts[0].trim();
                let title = parts[1].trim();
                let artist = parts[2].trim();
                let player_name_raw = parts[3].trim();
                let art_url = parts[4].trim();

                let playing = status.to_lowercase().contains("playing");

                if playing != last_play_state.get() {
                    last_play_state.set(playing);
                    if playing {
                        widgets.play_icon.set_icon_name(Some("media-playback-pause-symbolic"));
                    } else {
                        widgets.play_icon.set_icon_name(Some("media-playback-start-symbolic"));
                    }
                }

                let mut last_title_borrow = last_title.borrow_mut();
                if title != *last_title_borrow {
                    *last_title_borrow = title.to_string();
                    widgets.title_label.set_text(title);
                    
                    let display_artist = if artist.is_empty() { "Unknown Artist" } else { artist };
                    widgets.artist_label.set_text(display_artist);

                    let mut last_art_borrow = last_art_url.borrow_mut();
                    *last_art_borrow = art_url.to_string();

                    let mut art_loaded = false;
                    if !art_url.is_empty() {
                        if let Some(art_img) = load_album_art(art_url, 48) {
                            widgets.art_image.set_paintable(art_img.paintable().as_ref());
                            art_loaded = true;
                        }
                    }

                    if !art_loaded {
                        let app_icon_name = get_player_icon_name(player_name_raw);
                        let app_icon = archvnde_common::icon::get_icon_colored(&app_icon_name, 20, "#3b82f6");
                        widgets.art_image.set_paintable(app_icon.paintable().as_ref());
                    }
                }
            }
        } else {
            widgets.title_label.set_text("No media");
            widgets.artist_label.set_text("Unknown Artist");
            widgets.play_icon.set_icon_name(Some("media-playback-start-symbolic"));
            
            let default_icon = archvnde_common::icon::get_icon_colored("music", 20, "#3b82f6");
            widgets.art_image.set_paintable(default_icon.paintable().as_ref());
            
            last_title.borrow_mut().clear();
            last_art_url.borrow_mut().clear();
            last_play_state.set(false);
        }

        gtk4::glib::ControlFlow::Continue
    });
}
