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
use std::collections::HashMap;

fn get_current_volume() -> f64 {
    if let Ok(output) = std::process::Command::new("wpctl")
        .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(vol_str) = stdout.split_whitespace().nth(1) {
            if let Ok(vol) = vol_str.parse::<f64>() {
                return vol * 100.0;
            }
        }
    }
    if let Ok(output) = std::process::Command::new("pactl")
        .args(&["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(pos) = stdout.find('%') {
            let start = stdout[..pos].rfind(' ').unwrap_or(0);
            if let Ok(vol) = stdout[start..pos].trim().parse::<f64>() {
                return vol;
            }
        }
    }
    80.0
}

fn set_volume(val: f64) {
    let percent = val as i32;
    let _ = std::process::Command::new("wpctl")
        .args(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", percent)])
        .spawn();
    let _ = std::process::Command::new("pactl")
        .args(&["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", percent)])
        .spawn();
    let _ = std::process::Command::new("amixer")
        .args(&["set", "Master", &format!("{}%", percent)])
        .spawn();
}

fn get_current_brightness() -> f64 {
    if let Ok(output) = std::process::Command::new("brightnessctl")
        .args(&["-m"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let pct_str = parts[3].trim_end_matches('%');
                if let Ok(pct) = pct_str.parse::<f64>() {
                    return pct;
                }
            }
        }
    }
    if let Ok(output) = std::process::Command::new("light")
        .args(&["-G"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(pct) = stdout.trim().parse::<f64>() {
            return pct;
        }
    }
    60.0
}

fn set_brightness(val: f64) {
    let percent = val as i32;
    let _ = std::process::Command::new("brightnessctl")
        .args(&["set", &format!("{}%", percent)])
        .spawn();
    let _ = std::process::Command::new("light")
        .args(&["-S", &percent.to_string()])
        .spawn();
}

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

    status_content.append(&wifi_icon);
    status_content.append(&bluetooth_icon);

    // --- Battery: only show if a battery device exists in /sys/class/power_supply/ ---
    let battery_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
    battery_box.add_css_class("battery-box");

    fn get_battery_info() -> Option<(u8, bool)> {
        let power_dir = std::path::Path::new("/sys/class/power_supply");
        if !power_dir.exists() { return None; }
        for entry in std::fs::read_dir(power_dir).ok()?.flatten() {
            let path = entry.path();
            let type_path = path.join("type");
            if let Ok(kind) = std::fs::read_to_string(&type_path) {
                if kind.trim() == "Battery" {
                    let cap_str = std::fs::read_to_string(path.join("capacity")).ok()?;
                    let pct = cap_str.trim().parse::<u8>().ok()?;
                    let charging = std::fs::read_to_string(path.join("status"))
                        .map(|s| s.trim() == "Charging" || s.trim() == "Full")
                        .unwrap_or(false);
                    return Some((pct, charging));
                }
            }
        }
        None
    }

    if let Some((pct, charging)) = get_battery_info() {
        let battery_icon_name = if charging { "battery-charging" } else { "battery" };
        let battery_icon = archvnde_common::icon::get_icon(battery_icon_name, 14);
        battery_icon.add_css_class("status-icon");

        let battery_percent = gtk4::Label::new(Some(&format!("{}%", pct)));
        battery_percent.add_css_class("status-text");

        battery_box.append(&battery_icon);
        battery_box.append(&battery_percent);
        status_content.append(&battery_box);

        // Update battery every 30s
        let battery_box_c = battery_box.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_secs(30), move || {
            if let Some((new_pct, new_charging)) = get_battery_info() {
                // Update percentage label
                let mut child = battery_box_c.first_child();
                while let Some(widget) = child {
                    let next = widget.next_sibling();
                    if let Some(label) = widget.downcast_ref::<gtk4::Label>() {
                        label.set_text(&format!("{}%", new_pct));
                    }
                    if let Some(img) = widget.downcast_ref::<gtk4::Image>() {
                        let new_name = if new_charging { "battery-charging" } else { "battery" };
                        let new_icon = archvnde_common::icon::get_icon(new_name, 14);
                        if let Some(paintable) = new_icon.paintable() {
                            img.set_paintable(Some(&paintable));
                        }
                    }
                    child = next;
                }
            }
            gtk4::glib::ControlFlow::Continue
        });
    }
    // If no battery found, battery_box stays empty and is not appended → nothing shown

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

fn rebuild_control_center_contents(
    main_box: &gtk4::Box,
    on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>,
) {
    // 1. Remove all existing children
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }

    // 2. Append header
    main_box.append(&create_header_row());

    // 3. Append grid
    main_box.append(&create_control_center_grid());

    // 4. Append volume slider
    let initial_volume = get_current_volume();
    main_box.append(&create_slider_row(
        "volume",
        initial_volume,
        on_popover_toggled.clone(),
        |val| { set_volume(val); }
    ));

    // 5. Append brightness slider
    let initial_brightness = get_current_brightness();
    main_box.append(&create_slider_row(
        "brightness",
        initial_brightness,
        None,
        |val| { set_brightness(val); }
    ));

    // 6. Append disk monitor box
    main_box.append(&create_disk_list_box());
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
    archvnde_common::apply_theme_class(&q_win);
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

    let popover_active = Rc::new(std::cell::Cell::new(false));
    let popover_active_clone = popover_active.clone();
    let q_win_weak = q_win.downgrade();
    
    let motion_controller = gtk4::EventControllerMotion::new();
    main_box.add_controller(motion_controller.clone());
    let motion_c = motion_controller.clone();

    let on_popover_toggled = Rc::new(move |is_open: bool| {
        popover_active_clone.set(is_open);
        if !is_open {
            if !motion_c.contains_pointer() {
                if let Some(win) = q_win_weak.upgrade() {
                    win.close();
                }
            }
        }
    }) as Rc<dyn Fn(bool)>;

    let on_popover_toggled_opt = Some(on_popover_toggled.clone());
    rebuild_control_center_contents(&main_box, on_popover_toggled_opt.clone());

    if let Some(settings) = gtk4::Settings::default() {
        let main_box_c = main_box.clone();
        let on_popover_toggled_c = on_popover_toggled_opt.clone();
        settings.connect_gtk_application_prefer_dark_theme_notify(move |_| {
            rebuild_control_center_contents(&main_box_c, on_popover_toggled_c.clone());
        });
    }

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

    let popover_active_for_notify = popover_active.clone();
    q_win.connect_is_active_notify(move |win| {
        if !win.is_active() && !popover_active_for_notify.get() {
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

fn get_parent_drive(filesystem: &str) -> String {
    if filesystem.starts_with("/dev/sd") {
        if filesystem.len() >= 8 {
            return filesystem[0..8].to_string();
        }
    } else if filesystem.starts_with("/dev/nvme") {
        if let Some(p_idx) = filesystem.rfind('p') {
            if p_idx > 9 {
                return filesystem[0..p_idx].to_string();
            }
        }
    }
    filesystem.to_string()
}

fn format_size(kb: u64) -> String {
    let gb = kb as f64 / 1024.0 / 1024.0;
    if gb >= 1000.0 {
        format!("{:.1} TB", gb / 1024.0)
    } else {
        format!("{:.1} GB", gb)
    }
}

fn get_disk_list() -> Vec<DiskInfo> {
    let mut drive_map: HashMap<String, (u64, u64, u64)> = HashMap::new();
    let mut seen_partitions = std::collections::HashSet::new();

    let output = std::process::Command::new("df")
        .output();
    
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let filesystem = parts[0];
                if filesystem.starts_with("/dev/") {
                    if !seen_partitions.insert(filesystem.to_string()) {
                        continue;
                    }

                    let total_kb = parts[1].parse::<u64>().unwrap_or(0);
                    let used_kb = parts[2].parse::<u64>().unwrap_or(0);
                    let avail_kb = parts[3].parse::<u64>().unwrap_or(0);

                    let parent = get_parent_drive(filesystem);
                    let entry = drive_map.entry(parent).or_insert((0, 0, 0));
                    entry.0 += total_kb;
                    entry.1 += used_kb;
                    entry.2 += avail_kb;
                }
            }
        }
    }

    let mut list = Vec::new();
    for (drive, (total, used, avail)) in drive_map {
        if total > 0 {
            let percent = (used as f64 / total as f64) * 100.0;
            list.push(DiskInfo {
                filesystem: drive.clone(),
                size: format_size(total),
                used: format_size(used),
                avail: format_size(avail),
                percent,
                mount_point: drive,
            });
        }
    }

    list.sort_by(|a, b| a.filesystem.cmp(&b.filesystem));
    list
}

fn create_disk_list_box() -> gtk4::Box {
    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    card.add_css_class("control-disk-card");

    let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let disk_icon = archvnde_common::icon::get_icon_colored("server", 12, "#10b981");
    let title_label = gtk4::Label::new(Some(&archvnde_common::i18n::t("panel.storage_usage")));
    title_label.add_css_class("control-slider-title");
    
    title_row.append(&disk_icon);
    title_row.append(&title_label);
    card.append(&title_row);

    let list_container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    
    let disks = get_disk_list();
    if disks.is_empty() {
        let no_disks = gtk4::Label::new(Some(&archvnde_common::i18n::t("panel.no_storage")));
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
