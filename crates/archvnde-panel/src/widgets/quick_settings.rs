use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

/// Creates a Quick Settings toggle button that spawns/closes a dropdown-style settings overlay.
pub fn create_settings_button(app: &gtk4::Application) -> gtk4::Button {
    let settings_button = gtk4::Button::with_label("Wi-Fi | 100% ⚙");
    settings_button.add_css_class("panel-settings-btn");

    // Reference holder to manage the popup window lifetime
    let quick_settings_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));

    let qsw_clone = quick_settings_window.clone();
    let app_clone = app.clone();
    settings_button.connect_clicked(move |_| {
        let mut qsw_borrow = qsw_clone.borrow_mut();
        if let Some(existing_window) = qsw_borrow.take() {
            existing_window.close();
        } else {
            // Spawn a new overlay window for quick settings
            let q_win = gtk4::ApplicationWindow::new(&app_clone);
            q_win.init_layer_shell();
            q_win.set_layer(Layer::Overlay);

            // Position right under the top-right status bar
            q_win.set_anchor(Edge::Top, true);
            q_win.set_anchor(Edge::Right, true);
            q_win.set_margin(Edge::Top, 45);
            q_win.set_margin(Edge::Right, 15);
            q_win.set_default_size(300, 350);

            q_win.add_css_class("quick-settings-window");

            main_box.set_margin_start(15);
            main_box.set_margin_end(15);
            main_box.set_margin_top(15);
            main_box.set_margin_bottom(15);

            let title = gtk4::Label::new(Some("Quick Settings"));
            title.add_css_class("quick-settings-title");
            title.set_xalign(0.0);
            main_box.append(&title);

            // Grid of toggle buttons
            let grid = gtk4::Grid::new();
            grid.set_row_spacing(10);
            grid.set_column_spacing(10);

            let wifi_btn = gtk4::Button::with_label("Wi-Fi");
            wifi_btn.add_css_class("quick-tile");
            wifi_btn.add_css_class("active");
            grid.attach(&wifi_btn, 0, 0, 1, 1);

            let bt_btn = gtk4::Button::with_label("Bluetooth");
            bt_btn.add_css_class("quick-tile");
            grid.attach(&bt_btn, 1, 0, 1, 1);

            let dark_btn = gtk4::Button::with_label("Dark Mode");
            dark_btn.add_css_class("quick-tile");
            dark_btn.add_css_class("active");
            grid.attach(&dark_btn, 0, 1, 1, 1);

            let night_btn = gtk4::Button::with_label("Night Light");
            night_btn.add_css_class("quick-tile");
            grid.attach(&night_btn, 1, 1, 1, 1);

            main_box.append(&grid);

            // Volume slider
            let volume_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            let vol_label = gtk4::Label::new(Some("Vol"));
            let vol_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
            vol_scale.set_value(80.0);
            vol_scale.set_hexpand(true);
            volume_box.append(&vol_label);
            volume_box.append(&vol_scale);
            main_box.append(&volume_box);

            // Brightness slider
            let brightness_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            let bri_label = gtk4::Label::new(Some("Bri"));
            let bri_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
            bri_scale.set_value(60.0);
            bri_scale.set_hexpand(true);
            brightness_box.append(&bri_label);
            brightness_box.append(&bri_scale);
            main_box.append(&brightness_box);

            // Power actions
            let power_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            let power_off = gtk4::Button::with_label("Power Off");
            power_off.add_css_class("quick-tile");
            let logout = gtk4::Button::with_label("Log Out");
            logout.add_css_class("quick-tile");
            power_box.append(&power_off);
            power_box.append(&logout);
            main_box.append(&power_box);

            q_win.set_child(Some(&main_box));

            let qsw_inner = qsw_clone.clone();
            q_win.connect_close_request(move |_| {
                *qsw_inner.borrow_mut() = None;
                glib::Propagation::Proceed
            });

            q_win.present();
            *qsw_borrow = Some(q_win);
        }
    });

    settings_button
}
