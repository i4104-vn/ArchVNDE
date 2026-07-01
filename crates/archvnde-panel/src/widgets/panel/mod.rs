pub mod toggle_grid;
pub mod sliders;
pub mod power_actions;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use toggle_grid::create_quick_settings_grid;
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

    // --- 1. Status indicators button ---
    let status_button = gtk4::Button::new();
    status_button.add_css_class("panel-status-btn");

    let status_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);

    // Bluetooth icon
    let bluetooth_icon = archvnde_common::icon::get_icon("bluetooth", 14);
    bluetooth_icon.add_css_class("status-icon");

    // Wi-Fi icon
    let wifi_icon = archvnde_common::icon::get_icon("wifi", 14);
    wifi_icon.add_css_class("status-icon");

    // Battery with percentage
    let battery_icon = archvnde_common::icon::get_icon("battery", 14);
    battery_icon.add_css_class("status-icon");
    let battery_percent = gtk4::Label::new(Some("100%"));
    battery_percent.add_css_class("status-text");

    status_content.append(&wifi_icon);
    status_content.append(&bluetooth_icon);
    status_content.append(&battery_icon);
    status_content.append(&battery_percent);

    status_button.set_child(Some(&status_content));

    // Toggle Quick Settings
    let qsw_clone = quick_settings_window.clone();
    let cw_clone = calendar_window.clone();
    let app_clone = app.clone();
    status_button.connect_clicked(move |_| {
        // Close calendar if open
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
            let q_win = gtk4::ApplicationWindow::new(&app_clone);
            q_win.init_layer_shell();
            q_win.set_layer(Layer::Overlay);
            q_win.set_keyboard_mode(KeyboardMode::OnDemand);

            q_win.set_anchor(Edge::Top, true);
            q_win.set_anchor(Edge::Right, true);
            q_win.set_margin(Edge::Top, 10);
            q_win.set_margin(Edge::Right, 12);
            q_win.set_default_size(360, 360);
            q_win.add_css_class("quick-settings-window");

            let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
            main_box.add_css_class("quick-settings-box");
            main_box.set_valign(gtk4::Align::Start);

            // 1. Header with Title & Circle Actions
            main_box.append(&create_header_row());

            // 2. Volume control slider
            main_box.append(&create_slider_row("volume", 80.0, |val| {
                println!("Volume changed: {}%", val as i32);
            }));

            // 3. Brightness control slider
            main_box.append(&create_slider_row("brightness", 60.0, |val| {
                println!("Brightness changed: {}%", val as i32);
            }));

            // 4. Grid toggles
            main_box.append(&create_quick_settings_grid());

            q_win.set_child(Some(&main_box));

            let is_animating = Rc::new(std::cell::Cell::new(false));
            let is_animating_clone = is_animating.clone();
            let qsw_inner = qsw_clone.clone();
            let q_win_clone = q_win.clone();
            let main_box_clone = main_box.clone();
            q_win.connect_close_request(move |_| {
                if is_animating_clone.get() {
                    return glib::Propagation::Proceed;
                }
                is_animating_clone.set(true);
                let qsw_inner_cb = qsw_inner.clone();
                let q_win_cb = q_win_clone.clone();
                archvnde_common::animation::css_genie_out(
                    main_box_clone.upcast_ref(),
                    280,
                    move || {
                        if let Ok(mut borrow) = qsw_inner_cb.try_borrow_mut() {
                            *borrow = None;
                        }
                        q_win_cb.destroy();
                    }
                );
                glib::Propagation::Stop
            });

            q_win.present();
            archvnde_common::animation::css_genie_in(main_box.upcast_ref());
            if let Ok(mut borrow) = qsw_clone.try_borrow_mut() {
                *borrow = Some(q_win);
            }
        }
    });

    // --- 2. Separator line ---
    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");

    // --- 3. Clock widget button ---
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
