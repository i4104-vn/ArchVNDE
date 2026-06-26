use gtk4::prelude::*;

fn create_tile_row(
    icon_name: &str,
    title: &str,
    subtitle: &str,
    is_active: bool,
    active_class: &str,
    on_click: Option<impl Fn() + 'static>,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-tile-row");
    if is_active {
        btn.add_css_class(active_class);
    }
    btn.set_hexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_box.set_valign(gtk4::Align::Center);

    // Icon Circle wrapper
    let circle = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    circle.add_css_class("control-icon-circle");
    if is_active {
        circle.add_css_class("active");
    }
    
    let initial_color = if is_active { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
    let icon_widget = archvnde_common::icon::get_icon_colored(icon_name, 14, initial_color);
    circle.append(&icon_widget);
    main_box.append(&circle);

    // Text box
    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
    let title_label = gtk4::Label::new(Some(title));
    title_label.set_xalign(0.0);
    title_label.add_css_class("tile-title");

    let sub_label = gtk4::Label::new(Some(subtitle));
    sub_label.set_xalign(0.0);
    sub_label.add_css_class("tile-subtitle");

    text_box.append(&title_label);
    text_box.append(&sub_label);
    main_box.append(&text_box);

    btn.set_child(Some(&main_box));

    let act_class = active_class.to_string();
    let icon_name_str = icon_name.to_string();
    let circle_clone = circle.clone();
    let icon_widget_clone = icon_widget.clone();

    let on_click_opt = std::rc::Rc::new(std::cell::RefCell::new(on_click));

    btn.connect_clicked(move |b| {
        let is_now_active = if b.has_css_class(&act_class) {
            b.remove_css_class(&act_class);
            circle_clone.remove_css_class("active");
            false
        } else {
            b.add_css_class(&act_class);
            circle_clone.add_css_class("active");
            true
        };

        let color = if is_now_active { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
        let new_img = archvnde_common::icon::get_icon_colored(&icon_name_str, 14, color);
        if let Some(paintable) = new_img.paintable() {
            icon_widget_clone.set_paintable(Some(&paintable));
        }

        if let Some(ref cb) = *on_click_opt.borrow() {
            cb();
        }
    });

    btn
}

pub fn create_left_box_toggles() -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    container.add_css_class("control-left-toggles-box");
    container.set_valign(gtk4::Align::Fill);
    container.set_vexpand(true);

    let wifi_btn = create_tile_row("wifi", "Network", "Connected", true, "active", None::<fn()>);
    let bt_btn = create_tile_row("bluetooth", "Bluetooth", "Not Connected", false, "active", None::<fn()>);
    
    let settings_btn = create_tile_row(
        "settings",
        "Settings",
        "System Settings",
        true,
        "active-light",
        Some(|| {
            let _ = std::process::Command::new("archvnde-launcher").spawn();
        }),
    );

    container.append(&wifi_btn);
    container.append(&bt_btn);
    container.append(&settings_btn);
    container
}

pub fn create_dnd_tile() -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-dnd-tile");
    btn.set_hexpand(true);
    btn.set_valign(gtk4::Align::Fill);
    btn.set_vexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_box.set_valign(gtk4::Align::Center);

    let circle = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    circle.add_css_class("control-icon-circle");

    let icon_widget = archvnde_common::icon::get_icon_colored("bell", 14, "rgba(255, 255, 255, 0.7)");
    circle.append(&icon_widget);
    main_box.append(&circle);

    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
    let title_label = gtk4::Label::new(Some("Do Not Disturb"));
    title_label.set_xalign(0.0);
    title_label.add_css_class("tile-title");

    let sub_label = gtk4::Label::new(Some("Off"));
    sub_label.set_xalign(0.0);
    sub_label.add_css_class("tile-subtitle");

    text_box.append(&title_label);
    text_box.append(&sub_label);
    main_box.append(&text_box);

    btn.set_child(Some(&main_box));

    let circle_clone = circle.clone();
    let icon_widget_clone = icon_widget.clone();
    let sub_label_clone = sub_label.clone();

    btn.connect_clicked(move |b| {
        if b.has_css_class("active") {
            b.remove_css_class("active");
            circle_clone.remove_css_class("active");
            sub_label_clone.set_text("Off");
            let new_img = archvnde_common::icon::get_icon_colored("bell", 14, "rgba(255, 255, 255, 0.7)");
            if let Some(paintable) = new_img.paintable() {
                icon_widget_clone.set_paintable(Some(&paintable));
            }
        } else {
            b.add_css_class("active");
            circle_clone.add_css_class("active");
            sub_label_clone.set_text("On");
            let new_img = archvnde_common::icon::get_icon_colored("bell-off", 14, "#ffffff");
            if let Some(paintable) = new_img.paintable() {
                icon_widget_clone.set_paintable(Some(&paintable));
            }
        }
    });

    btn
}

pub fn create_small_square_tile(icon_name: &str, text: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-square-tile");
    btn.set_hexpand(true);
    btn.set_valign(gtk4::Align::Fill);
    btn.set_vexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.set_valign(gtk4::Align::Center);
    main_box.set_halign(gtk4::Align::Center);

    let icon_widget = archvnde_common::icon::get_icon_colored(icon_name, 16, "rgba(255, 255, 255, 0.8)");
    icon_widget.set_halign(gtk4::Align::Center);

    let label = gtk4::Label::new(Some(text));
    label.add_css_class("control-square-label");
    label.set_halign(gtk4::Align::Center);

    main_box.append(&icon_widget);
    main_box.append(&label);

    btn.set_child(Some(&main_box));

    let icon_name_str = icon_name.to_string();
    let icon_widget_clone = icon_widget.clone();

    btn.connect_clicked(move |b| {
        if b.has_css_class("active") {
            b.remove_css_class("active");
            let new_img = archvnde_common::icon::get_icon_colored(&icon_name_str, 16, "rgba(255, 255, 255, 0.8)");
            if let Some(paintable) = new_img.paintable() {
                icon_widget_clone.set_paintable(Some(&paintable));
            }
        } else {
            b.add_css_class("active");
            let new_img = archvnde_common::icon::get_icon_colored(&icon_name_str, 16, "#ffffff");
            if let Some(paintable) = new_img.paintable() {
                icon_widget_clone.set_paintable(Some(&paintable));
            }
        }
    });

    btn
}

pub fn create_control_center_grid() -> gtk4::Grid {
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);

    let left_box = create_left_box_toggles();
    grid.attach(&left_box, 0, 0, 1, 2);

    let dnd_btn = create_dnd_tile();
    grid.attach(&dnd_btn, 1, 0, 1, 1);

    let small_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    small_box.set_homogeneous(true);
    small_box.set_valign(gtk4::Align::Fill);
    small_box.set_vexpand(true);

    let kde_btn = create_small_square_tile("gsconnect", "KDE\nConnect");
    let night_btn = create_small_square_tile("night-light", "Night\nColor");

    small_box.append(&kde_btn);
    small_box.append(&night_btn);
    grid.attach(&small_box, 1, 1, 1, 1);

    grid
}
