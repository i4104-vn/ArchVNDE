use gtk4::prelude::*;

pub fn create_slider_row(icon: &str, initial_val: f64, on_changed: impl Fn(f64) + 'static) -> gtk4::Box {
    let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    box_layout.set_margin_top(4);

    let label = gtk4::Label::new(Some(icon));
    label.add_css_class("slider-label");

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
    scale.set_value(initial_val);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    scale.connect_value_changed(move |s| {
        on_changed(s.value());
    });

    box_layout.append(&label);
    box_layout.append(&scale);
    box_layout
}
