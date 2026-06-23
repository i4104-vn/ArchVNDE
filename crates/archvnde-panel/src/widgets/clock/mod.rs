use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

/// Creates and returns a clock button widget that updates every second and
/// spawns a centered, glassmorphic calendar popup dropdown when clicked.
pub fn create_clock_widget(app: &gtk4::Application) -> gtk4::Button {
    let clock_button = gtk4::Button::new();
    clock_button.add_css_class("panel-clock-btn");

    let clock_label = gtk4::Label::new(None);
    clock_label.add_css_class("panel-clock");
    clock_button.set_child(Some(&clock_label));

    let update_clock = {
        let clock_label = clock_label.clone();
        move || {
            let now = chrono::Local::now();
            let time_str = format!("26°C  |  {}", now.format("%a %b %d  |  %I:%M %p").to_string().to_uppercase());
            clock_label.set_text(&time_str);
            glib::ControlFlow::Continue
        }
    };
    update_clock(); // Run initially
    glib::timeout_add_local(std::time::Duration::from_secs(1), update_clock);

    let calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));
    let cw_clone = calendar_window.clone();
    let app_clone = app.clone();

    clock_button.connect_clicked(move |_| {
        let existing = {
            let borrow = cw_clone.borrow();
            borrow.clone()
        };
        if let Some(existing_window) = existing {
            existing_window.close();
        } else {
            let c_win = gtk4::ApplicationWindow::new(&app_clone);
            c_win.init_layer_shell();
            c_win.set_layer(Layer::Overlay);

            // Center horizontally by anchoring to Top but leaving Left/Right unanchored
            c_win.set_anchor(Edge::Top, true);
            c_win.set_margin(Edge::Top, 40);
            c_win.set_default_size(320, 360);
            c_win.add_css_class("calendar-window");

            let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
            main_box.add_css_class("calendar-box");
            main_box.set_valign(gtk4::Align::Start);

            // 1. Header: Big clock & Date
            let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            header_box.set_halign(gtk4::Align::Center);

            let big_time = gtk4::Label::new(None);
            big_time.add_css_class("calendar-header-time");
            
            let big_date = gtk4::Label::new(None);
            big_date.add_css_class("calendar-header-date");

            // Update header time/date initial values
            let now = chrono::Local::now();
            big_time.set_text(&now.format("%I:%M %p").to_string());
            big_date.set_text(&now.format("%A, %B %d").to_string());

            // Simple timer to update the header time every second while calendar is open
            let bt_clone = big_time.clone();
            let bd_clone = big_date.clone();
            let update_header = move || {
                let current_now = chrono::Local::now();
                bt_clone.set_text(&current_now.format("%I:%M %p").to_string());
                bd_clone.set_text(&current_now.format("%A, %B %d").to_string());
                glib::ControlFlow::Continue
            };
            glib::timeout_add_local(std::time::Duration::from_secs(1), update_header);

            header_box.append(&big_time);
            header_box.append(&big_date);
            main_box.append(&header_box);

            // 2. Divider line
            let divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            divider.add_css_class("calendar-divider");
            main_box.append(&divider);

            // 3. Calendar widget
            let calendar = gtk4::Calendar::new();
            calendar.add_css_class("calendar-widget");
            main_box.append(&calendar);

            c_win.set_child(Some(&main_box));

            let cw_inner = cw_clone.clone();
            c_win.connect_close_request(move |_| {
                if let Ok(mut borrow) = cw_inner.try_borrow_mut() {
                    *borrow = None;
                }
                glib::Propagation::Proceed
            });

            c_win.present();
            archvnde_animation::slide_in(main_box.upcast_ref(), archvnde_animation::SlideDirection::Down, 10, 220);
            if let Ok(mut borrow) = cw_clone.try_borrow_mut() {
                *borrow = Some(c_win);
            }
        }
    });

    clock_button
}
