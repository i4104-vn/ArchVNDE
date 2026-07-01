use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
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
            c_win.set_keyboard_mode(KeyboardMode::OnDemand);

            // Anchor to Top-Right to display calendar dropdown on the right side
            c_win.set_anchor(Edge::Top, true);
            c_win.set_anchor(Edge::Bottom, true);
            c_win.set_anchor(Edge::Right, true);
            c_win.set_margin(Edge::Top, 10);
            c_win.set_margin(Edge::Bottom, 10);
            c_win.set_margin(Edge::Right, 12);
            c_win.set_default_size(360, -1);
            c_win.add_css_class("calendar-window");

            let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
            main_box.add_css_class("calendar-box");
            main_box.set_vexpand(true);
            main_box.set_valign(gtk4::Align::Fill);

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

            // 4. Divider line
            let divider2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            divider2.add_css_class("calendar-divider");
            main_box.append(&divider2);

            // 5. Notifications Header
            let notif_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            notif_header.set_hexpand(true);

            let notif_title = gtk4::Label::new(Some("Thông báo"));
            notif_title.add_css_class("notif-panel-title");
            notif_title.set_halign(gtk4::Align::Start);
            notif_title.set_hexpand(true);

            let clear_btn = gtk4::Button::with_label("Xóa tất cả");
            clear_btn.add_css_class("clear-all-btn");
            clear_btn.set_halign(gtk4::Align::End);

            notif_header.append(&notif_title);
            notif_header.append(&clear_btn);
            main_box.append(&notif_header);

            // 6. Notifications Scrolled Window & List Box
            let scrolled_win = gtk4::ScrolledWindow::new();
            scrolled_win.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
            scrolled_win.set_vexpand(true);
            scrolled_win.set_hexpand(true);
            scrolled_win.add_css_class("notif-scroll");

            let notif_stack = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            notif_stack.add_css_class("notif-stack-box");
            notif_stack.set_vexpand(true);
            notif_stack.set_valign(gtk4::Align::Start);

            scrolled_win.set_child(Some(&notif_stack));
            main_box.append(&scrolled_win);

            // 7. Render Notifications logic
            let render_notifications = {
                let notif_stack = notif_stack.clone();
                move || {
                    while let Some(child) = notif_stack.first_child() {
                        notif_stack.remove(&child);
                    }

                    let notifications = archvnde_island::widgets::notification::HISTORICAL_NOTIFICATIONS.with(|list| {
                        list.borrow().clone()
                    });

                    if notifications.is_empty() {
                        let empty_label = gtk4::Label::new(Some("Không có thông báo mới"));
                        empty_label.add_css_class("notif-empty-label");
                        empty_label.set_halign(gtk4::Align::Center);
                        empty_label.set_valign(gtk4::Align::Center);
                        empty_label.set_vexpand(true);
                        notif_stack.append(&empty_label);
                    } else {
                        for notif in notifications.iter().rev() {
                            let item_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                            item_box.add_css_class("notif-stack-item");

                            let icon_symbol = if notif.icon.is_empty() { "message" } else { &notif.icon };
                            let icon_widget = archvnde_common::icon::get_icon_colored(icon_symbol, 18, "#3b82f6");
                            icon_widget.add_css_class("notif-item-icon");

                            let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
                            text_box.set_hexpand(true);

                            let title_lbl = gtk4::Label::new(Some(&notif.title));
                            title_lbl.add_css_class("notif-item-title");
                            title_lbl.set_halign(gtk4::Align::Start);

                            let body_lbl = gtk4::Label::new(Some(&notif.body));
                            body_lbl.add_css_class("notif-item-body");
                            body_lbl.set_halign(gtk4::Align::Start);
                            body_lbl.set_wrap(true);
                            body_lbl.set_max_width_chars(28);

                            text_box.append(&title_lbl);
                            text_box.append(&body_lbl);

                            item_box.append(&icon_widget);
                            item_box.append(&text_box);

                            notif_stack.append(&item_box);
                        }
                    }
                }
            };

            // Initial render
            render_notifications();

            let clear_btn_render_clone = render_notifications.clone();
            clear_btn.connect_clicked(move |_| {
                archvnde_island::widgets::notification::HISTORICAL_NOTIFICATIONS.with(|list| {
                    list.borrow_mut().clear();
                });
                clear_btn_render_clone();
            });

            // Update timer for time, date and notification count change detection
            let last_notif_count = Rc::new(std::cell::Cell::new(
                archvnde_island::widgets::notification::HISTORICAL_NOTIFICATIONS.with(|list| list.borrow().len())
            ));

            let bt_clone = big_time.clone();
            let bd_clone = big_date.clone();
            let render_timer_clone = render_notifications.clone();
            let last_count_clone = last_notif_count.clone();
            let update_header = move || {
                let current_now = chrono::Local::now();
                bt_clone.set_text(&current_now.format("%I:%M %p").to_string());
                bd_clone.set_text(&current_now.format("%A, %B %d").to_string());

                let current_count = archvnde_island::widgets::notification::HISTORICAL_NOTIFICATIONS.with(|list| {
                    list.borrow().len()
                });
                if current_count != last_count_clone.get() {
                    last_count_clone.set(current_count);
                    render_timer_clone();
                }

                glib::ControlFlow::Continue
            };
            glib::timeout_add_local(std::time::Duration::from_secs(1), update_header);

            c_win.set_child(Some(&main_box));

            c_win.connect_is_active_notify(|win| {
                if !win.is_active() {
                    win.close();
                }
            });

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
                archvnde_common::animation::css_genie_out(
                    main_box_clone.upcast_ref(),
                    400,
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
            archvnde_common::animation::css_genie_in(main_box.upcast_ref());
            if let Ok(mut borrow) = cw_clone.try_borrow_mut() {
                *borrow = Some(c_win);
            }
        }
    });

    clock_button
}
