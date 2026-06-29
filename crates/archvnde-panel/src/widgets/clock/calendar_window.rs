use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use super::notifications;

pub fn show_calendar_window(
    app: &gtk4::Application,
    cw_clone: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::ApplicationWindow {
    let c_win = gtk4::ApplicationWindow::new(app);
    c_win.init_layer_shell();
    c_win.set_layer(Layer::Overlay);
    c_win.set_keyboard_mode(KeyboardMode::OnDemand);

    // Anchor to all 4 edges to cover the entire screen transparently
    c_win.set_anchor(Edge::Top, true);
    c_win.set_anchor(Edge::Bottom, true);
    c_win.set_anchor(Edge::Left, true);
    c_win.set_anchor(Edge::Right, true);
    c_win.add_css_class("calendar-window");

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    main_box.add_css_class("calendar-box");
    main_box.set_vexpand(true);
    main_box.set_valign(gtk4::Align::Fill);
    main_box.set_halign(gtk4::Align::End);
    main_box.set_width_request(360);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_end(12);

    // 1. Header: Date on left
    let top_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    top_row.add_css_class("calendar-top-row");

    let date_label = gtk4::Label::new(None);
    date_label.add_css_class("calendar-top-date");
    date_label.set_halign(gtk4::Align::Start);
    date_label.set_hexpand(true);

    top_row.append(&date_label);
    main_box.append(&top_row);

    // Dummy time label to satisfy setup_notifications_list signature without displaying it
    let dummy_time = gtk4::Label::new(None);

    // 3. Calendar widget
    let calendar = gtk4::Calendar::new();
    calendar.add_css_class("calendar-widget");
    main_box.append(&calendar);

    // 5. Notifications Header
    let notif_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    notif_header.set_hexpand(true);

    let notif_title = gtk4::Label::new(Some(&archvnde_common::i18n::t("panel.notifications")));
    notif_title.add_css_class("notif-panel-title");
    notif_title.set_halign(gtk4::Align::Start);
    notif_title.set_hexpand(true);

    let clear_btn = gtk4::Button::with_label(&archvnde_common::i18n::t("panel.clear_all"));
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

    // 7. Setup and Render Notifications list
    notifications::setup_notifications_list(&notif_stack, &clear_btn, &dummy_time, &date_label);

    c_win.set_child(Some(&main_box));

    // Dismiss when clicking outside the calendar box area
    let click_gesture = gtk4::GestureClick::new();
    let main_box_c = main_box.clone();
    let window_c = c_win.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let picked = window_c.pick(x, y, gtk4::PickFlags::DEFAULT);
        let inside = picked
            .map(|w| w.is_ancestor(&main_box_c) || w == main_box_c)
            .unwrap_or(false);
        if !inside {
            window_c.close();
        }
    });
    c_win.add_controller(click_gesture);

    // Handle closing when window loses focus
    let cw_inner = cw_clone.clone();
    c_win.connect_is_active_notify(move |win| {
        if !win.is_active() {
            win.close();
        }
    });

    let is_animating = Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();
    let cw_inner_cb_clone = cw_inner.clone();
    let c_win_clone = c_win.clone();
    let main_box_clone = main_box.clone();
    c_win.connect_close_request(move |_| {
        if is_animating_clone.get() {
            return glib::Propagation::Proceed;
        }
        is_animating_clone.set(true);
        if let Ok(mut borrow) = cw_inner_cb_clone.try_borrow_mut() {
            *borrow = None;
        }
        let h = main_box_clone.height().max(480);
        let c_win_cb = c_win_clone.clone();
        archvnde_common::animation::genie_out(
            main_box_clone.upcast_ref(),
            360,
            h,
            400,
            move || {
                c_win_cb.destroy();
            }
        );
        glib::Propagation::Stop
    });

    let (_, natural_size) = main_box.preferred_size();
    let target_height = if natural_size.height() > 20 { natural_size.height() } else { 480 };
    c_win.present();
    archvnde_common::animation::genie_in(main_box.upcast_ref(), 360, target_height, 400);

    c_win
}
