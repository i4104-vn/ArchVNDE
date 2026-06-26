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

    // 7. Setup and Render Notifications list
    notifications::setup_notifications_list(&notif_stack, &clear_btn, &big_time, &big_date);

    c_win.set_child(Some(&main_box));

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
        let cw_inner_cb = cw_inner_cb_clone.clone();
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

    c_win
}
