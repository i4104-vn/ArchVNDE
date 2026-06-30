use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::Cell;

pub fn create_slider_row(icon_name: &str, initial_val: f64, on_changed: impl Fn(f64) + 'static) -> gtk4::Box {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.add_css_class("control-slider-card");

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    let title_text = match icon_name {
        "volume" => "Volume",
        "brightness" => "Brightness",
        _ => "Slider",
    };
    let title_label = gtk4::Label::new(Some(title_text));
    title_label.add_css_class("control-slider-title");
    title_label.set_xalign(0.0);
    title_label.set_hexpand(true);

    let value_label = gtk4::Label::new(Some(&format!("{:.0}%", initial_val)));
    value_label.add_css_class("control-slider-value");
    value_label.set_xalign(1.0);

    header_box.append(&title_label);
    header_box.append(&value_label);

    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

    let icon_widget = archvnde_icon::get_icon(icon_name, 16);
    icon_widget.add_css_class("slider-icon");

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_value(initial_val);
    scale.set_hexpand(true);
    scale.set_draw_value(false);

    // Debounce: prevent spamming system commands on every pixel moved
    let last_source: Rc<Cell<Option<gtk4::glib::SourceId>>> = Rc::new(Cell::new(None));
    let on_changed = Rc::new(on_changed);

    scale.connect_value_changed(move |s| {
        let val = s.value();
        value_label.set_text(&format!("{:.0}%", val));

        // Cancel any pending debounce timer
        if let Some(id) = last_source.take() {
            id.remove();
        }

        let on_changed = on_changed.clone();
        let new_id = gtk4::glib::timeout_add_local_once(
            std::time::Duration::from_millis(80),
            move || { on_changed(val); }
        );
        last_source.set(Some(new_id));
    });

    row_box.append(&icon_widget);
    row_box.append(&scale);

    main_box.append(&header_box);
    main_box.append(&row_box);
    main_box
}
