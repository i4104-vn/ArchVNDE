pub mod toggle_grid;
pub mod sliders;
pub mod power_actions;
pub mod storage;
mod render;

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use toggle_grid::create_control_center_grid;
use sliders::create_slider_row;
use power_actions::create_header_row;
use storage::create_disk_list_box;

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
    let (status_box, status_button, separator) = render::build_status_indicators_ui();

    let app_clone = app.clone();
    let ccw_clone = control_center_window.clone();
    let cw_clone = calendar_window.clone();
    let lw_clone = launcher_window.clone();
    status_button.connect_clicked(move |_| {
        let launcher_active = { lw_clone.borrow().clone() };
        if let Some(win) = launcher_active {
            win.close();
        }

        let cal_active = { cw_clone.borrow().clone() };
        if let Some(win) = cal_active {
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
    let (q_win, main_box) = render::build_control_center_window_ui(app);

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

    // Dismiss when clicking outside the control center box area
    archvnde_common::window::setup_click_outside_dismiss(&q_win, &main_box);

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
