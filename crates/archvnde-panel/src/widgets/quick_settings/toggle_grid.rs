use gtk4::prelude::*;

fn create_toggle_tile(
    icon_name: &str,
    title: &str,
    subtitle: &str,
    is_active: bool,
    active_class: &str,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("quick-tile");
    if is_active {
        btn.add_css_class(active_class);
    }
    btn.set_hexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);
    main_box.set_margin_top(6);
    main_box.set_margin_bottom(6);

    // Load SVG icon from archvnde-icon crate with state-specific color
    let initial_color = if is_active && active_class == "active-light" {
        "#161a26"
    } else {
        "#ffffff"
    };
    let icon_widget = archvnde_icon::get_icon_colored(icon_name, 16, initial_color);
    icon_widget.add_css_class("tile-icon");
    main_box.append(&icon_widget);

    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
    text_box.set_hexpand(true);
    
    let title_label = gtk4::Label::new(Some(title));
    title_label.set_xalign(0.0);
    title_label.add_css_class("tile-title");

    let sub_label = gtk4::Label::new(Some(subtitle));
    sub_label.set_xalign(0.0);
    sub_label.add_css_class("tile-subtitle");

    text_box.append(&title_label);
    text_box.append(&sub_label);
    main_box.append(&text_box);

    let chevron = gtk4::Label::new(Some("›"));
    chevron.add_css_class("tile-chevron");
    main_box.append(&chevron);

    btn.set_child(Some(&main_box));

    let act_class = active_class.to_string();
    let icon_name_str = icon_name.to_string();
    let icon_widget_clone = icon_widget.clone();
    btn.connect_clicked(move |b| {
        let is_now_active = if b.has_css_class(&act_class) {
            b.remove_css_class(&act_class);
            false
        } else {
            b.add_css_class(&act_class);
            true
        };

        let color = if is_now_active && act_class == "active-light" {
            "#161a26"
        } else {
            "#ffffff"
        };
        let new_img = archvnde_icon::get_icon_colored(&icon_name_str, 16, color);
        if let Some(paintable) = new_img.paintable() {
            icon_widget_clone.set_paintable(Some(&paintable));
        }
    });

    btn
}

pub fn create_quick_settings_grid() -> gtk4::Grid {
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);

    // Row 1
    grid.attach(&create_toggle_tile("wifi", "Wi-Fi", "Connected", true, "active"), 0, 0, 1, 1);
    grid.attach(&create_toggle_tile("bluetooth", "Bluetooth", "On", true, "active"), 1, 0, 1, 1);

    // Row 2
    grid.attach(&create_toggle_tile("performance", "Performance", "Balanced", false, "active"), 0, 1, 1, 1);
    grid.attach(&create_toggle_tile("night-light", "Night Light", "Off", false, "active"), 1, 1, 1, 1);

    // Row 3
    grid.attach(&create_toggle_tile("dark-mode", "Dark Style", "Active", true, "active-light"), 0, 2, 1, 1);
    grid.attach(&create_toggle_tile("caffeine", "Caffeine", "Off", false, "active"), 1, 2, 1, 1);

    // Row 4
    grid.attach(&create_toggle_tile("gsconnect", "GSConnect", "Disconnected", false, "active"), 0, 3, 1, 1);
    grid.attach(&create_toggle_tile("privacy", "Privacy", "3 allowed", false, "active"), 1, 3, 1, 1);

    grid
}
