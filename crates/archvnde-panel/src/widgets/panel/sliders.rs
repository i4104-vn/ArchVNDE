use gtk4::prelude::*;

pub fn create_slider_row(icon_name: &str, initial_val: f64, on_changed: impl Fn(f64) + 'static) -> gtk4::Box {
    let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

    let icon_widget = archvnde_common::icon::get_icon(icon_name, 16);
    icon_widget.add_css_class("slider-icon");

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
    scale.set_value(initial_val);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    scale.connect_value_changed(move |s| {
        on_changed(s.value());
    });

    box_layout.append(&icon_widget);
    box_layout.append(&scale);
    box_layout
}
