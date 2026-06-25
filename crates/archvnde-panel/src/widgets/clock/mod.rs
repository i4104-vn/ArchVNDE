use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

/// Creates and returns a clock button widget that updates every second and
/// spawns a centered, glassmorphic calendar popup dropdown when clicked.
pub fn create_clock_widget(
    app: &gtk4::Application,
    quick_settings_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::Button {
    let clock_button = gtk4::Button::new();
    clock_button.add_css_class("panel-clock-btn");

    let clock_label = gtk4::Label::new(None);
    clock_label.add_css_class("panel-clock");
    clock_button.set_child(Some(&clock_label));

    let update_clock = {
        let clock_label = clock_label.clone();
        move || {
            let now = chrono::Local::now();
            let time_str = format!(
                "{}   {}",
                now.format("%d/%m").to_string(),
                now.format("%I:%M %p").to_string().to_uppercase()
            );
            clock_label.set_text(&time_str);
            glib::ControlFlow::Continue
        }
    };
    update_clock(); // Run initially
    glib::timeout_add_local(std::time::Duration::from_secs(1), update_clock);

    let cw_clone = calendar_window.clone();
    let qsw_clone = quick_settings_window.clone();
    let app_clone = app.clone();

    clock_button.connect_clicked(move |_| {
        // Close Quick Settings window if open
        let qs_win = {
            qsw_clone.borrow().clone()
        };
        if let Some(win) = qs_win {
            win.close();
        }

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

            // Anchor to Top-Right to display calendar dropdown on the right side
            c_win.set_anchor(Edge::Top, true);
            c_win.set_anchor(Edge::Right, true);
            c_win.set_margin(Edge::Top, 10);
            c_win.set_margin(Edge::Right, 12);
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

            let is_animating = Rc::new(std::cell::Cell::new(false));
            let is_animating_clone = is_animating.clone();
            let cw_inner = cw_clone.clone();
            let c_win_clone = c_win.clone();
            let main_box_clone = main_box.clone();
            c_win.connect_close_request(move |_| {
                if is_animating_clone.get() {
                    return glib::Propagation::Proceed;
                }
                is_animating_clone.set(true);
                let cw_inner_cb = cw_inner.clone();
                let c_win_cb = c_win_clone.clone();
                archvnde_common::animation::css_zoom_out_cb(
                    main_box_clone.upcast_ref(),
                    280,
                    move || {
                        if let Ok(mut borrow) = cw_inner_cb.try_borrow_mut() {
                            *borrow = None;
                        }
                        c_win_cb.destroy();
                    }
                );
                glib::Propagation::Stop
            });

            c_win.present();
            archvnde_common::animation::css_zoom_in(main_box.upcast_ref());
            if let Ok(mut borrow) = cw_clone.try_borrow_mut() {
                *borrow = Some(c_win);
            }
        }
    });

    clock_button
}
