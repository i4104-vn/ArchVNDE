use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_settings_button(app: &gtk4::Application) -> gtk4::Button {
    let settings_button = gtk4::Button::with_label("Wi-Fi | 100% ⚙");
    settings_button.add_css_class("panel-settings-btn");

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

            main_box.append(&create_quick_settings_grid());

            main_box.append(&create_slider_row("🔊", 80.0, |val| {
                println!("Volume changed: {}%", val as i32);
            }));

            main_box.append(&create_slider_row("☀", 60.0, |val| {
                println!("Brightness changed: {}%", val as i32);
            }));

            main_box.append(&create_power_actions_row());

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

    settings_button
}

fn create_toggle_tile(label: &str, is_active: bool) -> gtk4::Button {
    let btn = gtk4::Button::with_label(label);
    btn.add_css_class("quick-tile");
    if is_active {
        btn.add_css_class("active");
    }
    btn.set_hexpand(true);
    btn.connect_clicked(move |b| {
        if b.has_css_class("active") {
            b.remove_css_class("active");
        } else {
            b.add_css_class("active");
        }
    });
    btn
}

fn create_quick_settings_grid() -> gtk4::Grid {
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);

    grid.attach(&create_toggle_tile("Wi-Fi", true), 0, 0, 1, 1);
    grid.attach(&create_toggle_tile("Bluetooth", false), 1, 0, 1, 1);
    grid.attach(&create_toggle_tile("Dark Mode", true), 0, 1, 1, 1);
    grid.attach(&create_toggle_tile("Night Light", false), 1, 1, 1, 1);

    grid
}

fn create_slider_row(icon: &str, initial_val: f64, on_changed: impl Fn(f64) + 'static) -> gtk4::Box {
    let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    box_layout.set_margin_top(4);

    let label = gtk4::Label::new(Some(icon));
    label.add_css_class("slider-label");

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
    scale.set_value(initial_val);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    scale.connect_value_changed(move |s| {
        on_changed(s.value());
    });

    box_layout.append(&label);
    box_layout.append(&scale);
    box_layout
}

fn create_power_actions_row() -> gtk4::Box {
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
    logout.add_css_class("logout-btn");
    logout.connect_clicked(move |_| {
        println!("Log Out requested...");
        if let Ok(user) = std::env::var("USER") {
            let _ = std::process::Command::new("loginctl").args(["terminate-user", &user]).spawn();
        }
    });

    power_box.append(&power_off);
    power_box.append(&logout);
    power_box
}
