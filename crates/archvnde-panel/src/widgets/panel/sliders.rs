use gtk4::prelude::*;

pub fn create_slider_row(icon_name: &str, initial_val: f64, on_changed: impl Fn(f64) + 'static) -> gtk4::Box {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.add_css_class("control-slider-card");

    let title_text = match icon_name {
        "volume" => "Volume",
        "brightness" => "Brightness",
        _ => "Slider",
    };
    let title_label = gtk4::Label::new(Some(title_text));
    title_label.add_css_class("control-slider-title");
    title_label.set_xalign(0.0);

    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

    let icon_widget = archvnde_icon::get_icon(icon_name, 16);
    icon_widget.add_css_class("slider-icon");

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
    scale.set_value(initial_val);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    scale.connect_value_changed(move |s| {
        on_changed(s.value());
    });

    row_box.append(&icon_widget);
    row_box.append(&scale);

    main_box.append(&title_label);
    main_box.append(&row_box);
    main_box
}
