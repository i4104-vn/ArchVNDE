pub mod toggle_grid;
pub mod sliders;
pub mod power_actions;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use toggle_grid::create_quick_settings_grid;
use sliders::create_slider_row;
use power_actions::create_header_row;

/// Creates a status indicator area with individually placed icons and labels.
/// These are passive display items, not clickable.
pub fn create_status_indicators() -> gtk4::Box {
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    status_box.add_css_class("status-indicators-box");
    status_box.set_valign(gtk4::Align::Center);

    // Language indicator
    let lang_label = gtk4::Label::new(Some("US"));
    lang_label.add_css_class("status-text");

    // Network speed
    let net_label = gtk4::Label::new(Some("844 B/s"));
    net_label.add_css_class("status-text");

    // Bluetooth icon
    let bluetooth_icon = archvnde_icon::get_icon("bluetooth", 14);
    bluetooth_icon.add_css_class("status-icon");

    // Wi-Fi icon
    let wifi_icon = archvnde_icon::get_icon("wifi", 14);
    wifi_icon.add_css_class("status-icon");

    // Battery with percentage
    let battery_icon = archvnde_icon::get_icon("battery", 14);
    battery_icon.add_css_class("status-icon");
    let battery_percent = gtk4::Label::new(Some("100%"));
    battery_percent.add_css_class("status-text");

    status_box.append(&lang_label);
    status_box.append(&net_label);
    status_box.append(&bluetooth_icon);
    status_box.append(&wifi_icon);
    status_box.append(&battery_icon);
    status_box.append(&battery_percent);

    status_box
}

/// Creates a clickable settings trigger button (gear + power) that opens Quick Settings.
pub fn create_settings_button(app: &gtk4::Application) -> gtk4::Box {
    let action_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    action_box.add_css_class("action-buttons-box");
    action_box.set_valign(gtk4::Align::Center);

    // Settings gear button — opens quick settings popup
    let settings_button = gtk4::Button::new();
    settings_button.add_css_class("panel-action-btn");
    let settings_icon = archvnde_icon::get_icon("settings", 14);
    settings_button.set_child(Some(&settings_icon));

    // Power button on the bar
    let power_button = gtk4::Button::new();
    power_button.add_css_class("panel-action-btn");
    power_button.add_css_class("power-btn");
    let power_icon = archvnde_icon::get_icon("power", 14);
    power_button.set_child(Some(&power_icon));
    power_button.connect_clicked(|_| {
        crate::widgets::power::trigger_shutdown();
    });

    let quick_settings_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));

    let qsw_clone = quick_settings_window.clone();
    let app_clone = app.clone();
    settings_button.connect_clicked(move |_| {
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
            q_win.set_blur_allowed(true);

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

            let qsw_inner = qsw_clone.clone();
            q_win.connect_close_request(move |_| {
                if let Ok(mut borrow) = qsw_inner.try_borrow_mut() {
                    *borrow = None;
                }
                glib::Propagation::Proceed
            });

            q_win.present();
            archvnde_animation::slide_in(main_box.upcast_ref(), archvnde_animation::SlideDirection::Down, 10, 220);
            if let Ok(mut borrow) = qsw_clone.try_borrow_mut() {
                *borrow = Some(q_win);
            }
        }
    });

    action_box.append(&settings_button);
    action_box.append(&power_button);

    action_box
}
