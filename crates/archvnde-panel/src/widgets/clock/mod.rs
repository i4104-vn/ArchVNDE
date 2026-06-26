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
            let expanded_apps = Rc::new(RefCell::new(std::collections::HashSet::<String>::new()));
            let render_notifications_holder: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

            let render_notifications = {
                let notif_stack = notif_stack.clone();
                let expanded_apps = expanded_apps.clone();
                let holder = render_notifications_holder.clone();
                move || {
                    let render_notifications_rc = holder.borrow().as_ref().unwrap().clone();

                    notif_stack.set_opacity(1.0);
                    notif_stack.set_margin_top(0);
                    notif_stack.set_margin_bottom(0);
                    notif_stack.set_margin_start(0);
                    notif_stack.set_margin_end(0);

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
                        // Group notifications by app/icon name
                        let mut grouped = std::collections::HashMap::<String, Vec<archvnde_island::widgets::notification::ActiveNotification>>::new();
                        let mut app_order = Vec::new();

                        for notif in notifications.iter() {
                            let app_key = if notif.icon.is_empty() { "system".to_string() } else { notif.icon.to_lowercase() };
                            if !grouped.contains_key(&app_key) {
                                app_order.push(app_key.clone());
                            }
                            grouped.entry(app_key).or_default().push(notif.clone());
                        }

                        app_order.reverse(); // Newest app group on top

                        for app_key in app_order {
                            let list = &grouped[&app_key];
                            let display_app_name = if app_key == "system" {
                                "Hệ thống".to_string()
                            } else {
                                let mut chars = app_key.chars();
                                match chars.next() {
                                    None => String::new(),
                                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                                }
                            };

                            let is_expanded = expanded_apps.borrow().contains(&app_key);

                            let group_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                            group_container.add_css_class("notif-group-container");

                            if is_expanded {
                                // Header for the expanded section (app name, icon, collapse trigger)
                                let group_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                                group_header.add_css_class("notif-group-header");

                                let name = if app_key == "system" { "preferences-system" } else { &app_key };
                                let icon_widget = if name.starts_with('/') {
                                    gtk4::Image::from_file(name)
                                } else {
                                    gtk4::Image::from_icon_name(name)
                                };
                                icon_widget.set_pixel_size(18);
                                icon_widget.add_css_class("notif-item-icon");

                                let title_lbl = gtk4::Label::new(Some(&display_app_name));
                                title_lbl.add_css_class("notif-item-title");
                                title_lbl.set_halign(gtk4::Align::Start);
                                title_lbl.set_hexpand(true);

                                let chevron = archvnde_common::icon::get_icon_colored("chevron-up", 12, "rgba(255, 255, 255, 0.4)");

                                group_header.append(&icon_widget);
                                group_header.append(&title_lbl);
                                group_header.append(&chevron);
                                group_container.append(&group_header);

                                // List of notifications
                                let sub_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
                                sub_box.add_css_class("notif-sub-box");

                                for notif in list.iter().rev() {
                                    let item_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                                    item_box.add_css_class("notif-stack-item");

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

                                    let time_str = format_elapsed_time(notif.timestamp);
                                    let time_lbl = gtk4::Label::new(Some(&time_str));
                                    time_lbl.add_css_class("notif-item-sub-time");
                                    time_lbl.set_halign(gtk4::Align::End);
                                    time_lbl.set_valign(gtk4::Align::Center);
 
                                    item_box.append(&text_box);
                                    item_box.append(&time_lbl);
                                    sub_box.append(&item_box);
                                }

                                group_container.append(&sub_box);

                                // Expand/Collapse click gesture
                                let click_gesture = gtk4::GestureClick::new();
                                let ea_c = expanded_apps.clone();
                                let ak_c = app_key.clone();
                                let render_c = render_notifications_rc.clone();
                                let sub_box_c = sub_box.clone();
                                click_gesture.connect_pressed(move |_, _, _, _| {
                                    let ea_cb = ea_c.clone();
                                    let ak_cb = ak_c.clone();
                                    let render_cb = render_c.clone();
                                    archvnde_common::animation::slide_out_cb(
                                        sub_box_c.upcast_ref(),
                                        archvnde_common::animation::SlideDirection::Up,
                                        15,
                                        200,
                                        false,
                                        move || {
                                            ea_cb.borrow_mut().remove(&ak_cb);
                                            render_cb();
                                        }
                                    );
                                });
                                group_header.add_controller(click_gesture);

                                // Slide in the sub_box when first expanded
                                archvnde_common::animation::slide_in(
                                    sub_box.upcast_ref(),
                                    archvnde_common::animation::SlideDirection::Down,
                                    15,
                                    250,
                                );
                            } else {
                                // Collapsed representation: Show latest notification from this app with badge count
                                let latest_notif = list.last().unwrap();

                                let main_item = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                                main_item.add_css_class("notif-stack-item");

                                let name = if app_key == "system" { "preferences-system" } else { &app_key };
                                let icon_widget = if name.starts_with('/') {
                                    gtk4::Image::from_file(name)
                                } else {
                                    gtk4::Image::from_icon_name(name)
                                };
                                icon_widget.set_pixel_size(18);
                                icon_widget.add_css_class("notif-item-icon");

                                let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
                                text_box.set_hexpand(true);

                                let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                                let app_title = gtk4::Label::new(Some(&display_app_name));
                                app_title.add_css_class("notif-item-title");
                                app_title.set_halign(gtk4::Align::Start);
                                header_box.append(&app_title);

                                let body_lbl = gtk4::Label::new(Some(&latest_notif.body));
                                body_lbl.add_css_class("notif-item-body");
                                body_lbl.set_halign(gtk4::Align::Start);
                                body_lbl.set_wrap(true);
                                body_lbl.set_max_width_chars(28);

                                text_box.append(&header_box);
                                text_box.append(&body_lbl);

                                let right_widget = if list.len() > 1 {
                                    let badge = gtk4::Label::new(Some(&format!("{}", list.len())));
                                    badge.add_css_class("notif-count-badge");
                                    badge.add_css_class("notif-item-sub-time");
                                    badge.set_halign(gtk4::Align::End);
                                    badge.set_valign(gtk4::Align::Center);
                                    badge.upcast::<gtk4::Widget>()
                                } else {
                                    let time_str = format_elapsed_time(latest_notif.timestamp);
                                    let time_lbl = gtk4::Label::new(Some(&time_str));
                                    time_lbl.add_css_class("notif-item-sub-time");
                                    time_lbl.set_halign(gtk4::Align::End);
                                    time_lbl.set_valign(gtk4::Align::Center);
                                    time_lbl.upcast::<gtk4::Widget>()
                                };

                                main_item.append(&icon_widget);
                                main_item.append(&text_box);
                                main_item.append(&right_widget);
                                group_container.append(&main_item);

                                // 3D Stack look if more than 1 notification
                                if list.len() > 1 {
                                    let layer1 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
                                    layer1.add_css_class("notif-stack-item-layered-1");
                                    let layer2 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
                                    layer2.add_css_class("notif-stack-item-layered-2");
                                    group_container.append(&layer2);
                                    group_container.append(&layer1);
                                }

                                // Click gesture to expand
                                let click_gesture = gtk4::GestureClick::new();
                                let ea_c = expanded_apps.clone();
                                let ak_c = app_key.clone();
                                let render_c = render_notifications_rc.clone();
                                click_gesture.connect_pressed(move |_, _, _, _| {
                                    ea_c.borrow_mut().insert(ak_c.clone());
                                    render_c();
                                });
                                group_container.add_controller(click_gesture);
                            }

                            notif_stack.append(&group_container);
                        }
                    }
                }
            };

            let render_rc = Rc::new(render_notifications);
            *render_notifications_holder.borrow_mut() = Some(render_rc.clone());

            // Initial render
            render_rc();

            let clear_btn_render_clone = render_rc.clone();
            let notif_stack_clear_clone = notif_stack.clone();
            clear_btn.connect_clicked(move |_| {
                let cb = clear_btn_render_clone.clone();
                archvnde_island::widgets::notification::HISTORICAL_NOTIFICATIONS.with(|list| {
                    list.borrow_mut().clear();
                });
                archvnde_common::animation::slide_out_cb(
                    notif_stack_clear_clone.upcast_ref(),
                    archvnde_common::animation::SlideDirection::Up,
                    20,
                    250,
                    false,
                    move || {
                        cb();
                    }
                );
            });

            // Update timer for time, date and notification count change detection
            let last_notif_count = Rc::new(std::cell::Cell::new(
                archvnde_island::widgets::notification::HISTORICAL_NOTIFICATIONS.with(|list| list.borrow().len())
            ));

            let bt_clone = big_time.clone();
            let bd_clone = big_date.clone();
            let render_timer_clone = render_rc.clone();
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

fn format_elapsed_time(instant: std::time::Instant) -> String {
    let secs = instant.elapsed().as_secs();
    if secs < 60 {
        "Vừa xong".to_string()
    } else if secs < 3600 {
        format!("{} phút trước", secs / 60)
    } else if secs < 86400 {
        format!("{} giờ trước", secs / 3600)
    } else {
        format!("{} ngày trước", secs / 86400)
    }
}
