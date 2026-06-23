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
            q_win.set_margin(Edge::Top, 46);
            q_win.set_margin(Edge::Right, 12);
            q_win.set_default_size(360, 380);

            q_win.add_css_class("quick-settings-window");

            let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
            main_box.add_css_class("quick-settings-box");

            let title = gtk4::Label::new(Some("Quick Settings"));
            title.add_css_class("quick-settings-title");
            title.set_xalign(0.0);
            main_box.append(&title);

            // ── Toggle tile grid ──
            let grid = gtk4::Grid::new();
            grid.set_row_spacing(10);
            grid.set_column_spacing(10);
            grid.set_row_homogeneous(true);
            grid.set_column_homogeneous(true);

            let wifi_btn = gtk4::Button::with_label("Wi-Fi");
            wifi_btn.add_css_class("quick-tile");
            wifi_btn.add_css_class("active");
            wifi_btn.set_hexpand(true);

            let bt_btn = gtk4::Button::with_label("Bluetooth");
            bt_btn.add_css_class("quick-tile");
            bt_btn.set_hexpand(true);

            let dark_btn = gtk4::Button::with_label("Dark Mode");
            dark_btn.add_css_class("quick-tile");
            dark_btn.add_css_class("active");
            dark_btn.set_hexpand(true);

            let night_btn = gtk4::Button::with_label("Night Light");
            night_btn.add_css_class("quick-tile");
            night_btn.set_hexpand(true);

            let toggle_active = |btn: &gtk4::Button| {
                btn.connect_clicked(move |b| {
                    if b.has_css_class("active") {
                        b.remove_css_class("active");
                    } else {
                        b.add_css_class("active");
                    }
                });
            };
            toggle_active(&wifi_btn);
            toggle_active(&bt_btn);
            toggle_active(&dark_btn);
            toggle_active(&night_btn);

            grid.attach(&wifi_btn, 0, 0, 1, 1);
            grid.attach(&bt_btn, 1, 0, 1, 1);
            grid.attach(&dark_btn, 0, 1, 1, 1);
            grid.attach(&night_btn, 1, 1, 1, 1);

            main_box.append(&grid);

            // ── Volume slider ──
            let volume_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            volume_box.set_margin_top(4);
            let vol_label = gtk4::Label::new(Some("🔊"));
            vol_label.add_css_class("slider-label");
            let vol_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
            vol_scale.set_value(80.0);
            vol_scale.set_hexpand(true);
            vol_scale.set_draw_value(false);
            vol_scale.connect_value_changed(move |scale| {
                println!("Volume changed: {}%", scale.value() as i32);
            });
            volume_box.append(&vol_label);
            volume_box.append(&vol_scale);
            main_box.append(&volume_box);

            // ── Brightness slider ──
            let brightness_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            let bri_label = gtk4::Label::new(Some("☀"));
            bri_label.add_css_class("slider-label");
            let bri_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
            bri_scale.set_value(60.0);
            bri_scale.set_hexpand(true);
            bri_scale.set_draw_value(false);
            bri_scale.connect_value_changed(move |scale| {
                println!("Brightness changed: {}%", scale.value() as i32);
            });
            brightness_box.append(&bri_label);
            brightness_box.append(&bri_scale);
            main_box.append(&brightness_box);

            // ── Power actions ──
            let power_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            power_box.set_margin_top(6);
            power_box.set_homogeneous(true);

            let power_off = gtk4::Button::with_label("⏻  Power Off");
            power_off.add_css_class("power-btn");
            power_off.connect_clicked(move |_| {
                println!("Power Off requested...");
                let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
            });
            
            let logout = gtk4::Button::with_label("↪  Log Out");
            logout.add_css_class("power-btn");
            logout.connect_clicked(move |_| {
                println!("Log Out requested...");
                if let Ok(user) = std::env::var("USER") {
                    let _ = std::process::Command::new("loginctl").args(["terminate-user", &user]).spawn();
                }
            });
            
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
            archvnde_animation::slide_in(q_win.upcast_ref(), archvnde_animation::SlideDirection::Down, 20, 250);
            *qsw_borrow = Some(q_win);
        }
    });

    settings_button
}
