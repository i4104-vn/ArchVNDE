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
            q_win.set_default_size(360, 360);
            q_win.add_css_class("quick-settings-window");

            let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
            main_box.add_css_class("quick-settings-box");

            // 1. Header with Title & Circle Actions
            main_box.append(&create_header_row());

            // 2. Volume control slider
            main_box.append(&create_slider_row("🔊", 80.0, |val| {
                println!("Volume changed: {}%", val as i32);
            }));

            // 3. Brightness control slider
            main_box.append(&create_slider_row("☀", 60.0, |val| {
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

    settings_button
}
