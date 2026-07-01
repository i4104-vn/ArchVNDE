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
/// Clicking the status button toggles Control Center; clicking the clock button toggles Calendar.
/// The two panels are mutually exclusive.
pub fn create_status_indicators(
    app: &gtk4::Application,
    control_center_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    launcher_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
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

    let ccw_clone = control_center_window.clone();
    let cw_clone = calendar_window.clone();
    let lw_clone = launcher_window.clone();
    let app_clone = app.clone();
    status_button.connect_clicked(move |_| {
        let cal_win = {
            cw_clone.borrow().clone()
        };
        if let Some(win) = cal_win {
            win.close();
        }
        
        let launch_win = {
            lw_clone.borrow().clone()
        };
        if let Some(win) = launch_win {
            win.close();
        }

        let existing = {
            let borrow = ccw_clone.borrow();
            borrow.clone()
        };
        if let Some(existing_window) = existing {
            existing_window.close();
        } else {
            let q_win = create_control_center_window(&app_clone, ccw_clone.clone());
            if let Ok(mut borrow) = ccw_clone.try_borrow_mut() {
                *borrow = Some(q_win);
            }
        }
    });

    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");

    let clock_button = crate::widgets::clock::create_clock_widget(
        app,
        control_center_window.clone(),
        calendar_window.clone(),
        launcher_window.clone(),
    );

    status_box.append(&status_button);
    status_box.append(&separator);
    status_box.append(&clock_button);

    status_box
}

/// Builds and maps a glassmorphic Control Center popup ApplicationWindow anchored
/// to the top-right corner. It binds volume and brightness sliders, grid toggles,
/// and registers Genie animations on close and map events.
fn create_control_center_window(
    app: &gtk4::Application,
    control_center_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::ApplicationWindow {
    use gtk4_layer_shell::{KeyboardMode, Layer, Edge};

    let q_win = gtk4::ApplicationWindow::new(app);
    q_win.init_layer_shell();
    q_win.set_layer(Layer::Overlay);
    q_win.set_keyboard_mode(KeyboardMode::OnDemand);

    // Anchor to all 4 edges to cover the entire screen transparently
    q_win.set_anchor(Edge::Top, true);
    q_win.set_anchor(Edge::Bottom, true);
    q_win.set_anchor(Edge::Left, true);
    q_win.set_anchor(Edge::Right, true);
    q_win.add_css_class("control-center-window");

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    main_box.add_css_class("control-center-box");
    main_box.set_halign(gtk4::Align::End);
    main_box.set_valign(gtk4::Align::Start);
    main_box.set_size_request(360, 480);
    main_box.set_margin_top(6);
    main_box.set_margin_end(12);

    main_box.append(&create_header_row());
    main_box.append(&create_control_center_grid());

    main_box.append(&create_slider_row("volume", 80.0, |val| {
        println!("Volume changed: {}%", val as i32);
    }));

    main_box.append(&create_slider_row("brightness", 60.0, |val| {
        println!("Brightness changed: {}%", val as i32);
    }));

    let disk_box = create_disk_list_box();
    main_box.append(&disk_box);

    q_win.set_child(Some(&main_box));

    // Dismiss when clicking outside the control center box area
    let click_gesture = gtk4::GestureClick::new();
    let main_box_c = main_box.clone();
    let window_c = q_win.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let picked = window_c.pick(x, y, gtk4::PickFlags::DEFAULT);
        let inside = picked
            .map(|w| w.is_ancestor(&main_box_c) || w == main_box_c)
            .unwrap_or(false);
        if !inside {
            window_c.close();
        }
    });
    q_win.add_controller(click_gesture);

    q_win.connect_is_active_notify(|win| {
        if !win.is_active() {
            win.close();
        }
    });

    let is_animating = Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();
    let ccw_inner = control_center_window.clone();
    let q_win_clone = q_win.clone();
    let main_box_clone = main_box.clone();
    q_win.connect_close_request(move |_| {
        if is_animating_clone.get() {
            return glib::Propagation::Stop;
        }
        is_animating_clone.set(true);
        if let Ok(mut borrow) = ccw_inner.try_borrow_mut() {
            *borrow = None;
        }
        let q_win_cb = q_win_clone.clone();
        archvnde_common::animation::genie_out(
            main_box_clone.upcast_ref(),
            360,
            480,
            450,
            move || {
                q_win_cb.destroy();
            }
        );
        glib::Propagation::Stop
    });

    q_win.present();
    archvnde_common::animation::genie_in(main_box.upcast_ref(), 360, 480, 450);

    q_win
}

#[derive(Clone, Debug)]
struct DiskInfo {
    filesystem: String,
    size: String,
    used: String,
    avail: String,
    percent: f64,
    mount_point: String,
}

fn get_disk_list() -> Vec<DiskInfo> {
    let mut list = Vec::new();
    let output = std::process::Command::new("df")
        .arg("-h")
        .output();
    
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let filesystem = parts[0];
                if filesystem.starts_with("/dev/") {
                    let size = parts[1].to_string();
                    let used = parts[2].to_string();
                    let avail = parts[3].to_string();
                    let pcent_str = parts[4].trim_end_matches('%');
                    let percent = pcent_str.parse::<f64>().unwrap_or(0.0);
                    let mount_point = parts[5].to_string();
                    
                    list.push(DiskInfo {
                        filesystem: filesystem.to_string(),
                        size,
                        used,
                        avail,
                        percent,
                        mount_point,
                    });
                }
            }
        }
    }
    list
}

fn create_disk_list_box() -> gtk4::Box {
    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    card.add_css_class("control-disk-card");

    let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let disk_icon = archvnde_common::icon::get_icon_colored("server", 12, "#10b981");
    let title_label = gtk4::Label::new(Some("Storage Usage"));
    title_label.add_css_class("control-slider-title");
    
    title_row.append(&disk_icon);
    title_row.append(&title_label);
    card.append(&title_row);

    let list_container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    
    let disks = get_disk_list();
    if disks.is_empty() {
        let no_disks = gtk4::Label::new(Some("No physical storage found"));
        no_disks.add_css_class("tile-subtitle");
        list_container.append(&no_disks);
    } else {
        for disk in disks.into_iter().take(3) {
            let disk_item = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
            disk_item.add_css_class("control-disk-item");

            let label_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            label_box.add_css_class("control-disk-title-box");

            let name_label = gtk4::Label::new(Some(&disk.mount_point));
            name_label.add_css_class("control-disk-name");
            name_label.set_hexpand(true);
            name_label.set_halign(gtk4::Align::Start);

            let usage_label = gtk4::Label::new(Some(&format!(
                "{} / {} ({:.0}%)",
                disk.used, disk.size, disk.percent
            )));
            usage_label.add_css_class("control-disk-usage");
            usage_label.set_halign(gtk4::Align::End);

            label_box.append(&name_label);
            label_box.append(&usage_label);

            let progress = gtk4::ProgressBar::new();
            progress.set_fraction(disk.percent / 100.0);
            progress.set_hexpand(true);

            disk_item.append(&label_box);
            disk_item.append(&progress);

            list_container.append(&disk_item);
        }
    }

    card.append(&list_container);
    card
}
