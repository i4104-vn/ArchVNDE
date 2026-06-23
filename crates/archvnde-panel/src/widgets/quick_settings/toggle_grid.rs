use gtk4::prelude::*;

fn create_toggle_tile(label: &str, is_active: bool) -> gtk4::Button {
    let btn = gtk4::Button::with_label(label);
    btn.add_css_class("quick-tile");
    if is_active {
        btn.add_css_class("active");
    }
    btn.set_hexpand(true);
    btn.connect_clicked(move |b| {
        if b.has_css_class("active") {
            b.remove_css_class("active");
        } else {
            b.add_css_class("active");
        }
    });
    btn
}

pub fn create_quick_settings_grid() -> gtk4::Grid {
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);

    grid.attach(&create_toggle_tile("Wi-Fi", true), 0, 0, 1, 1);
    grid.attach(&create_toggle_tile("Bluetooth", false), 1, 0, 1, 1);
    grid.attach(&create_toggle_tile("Dark Mode", true), 0, 1, 1, 1);
    grid.attach(&create_toggle_tile("Night Light", false), 1, 1, 1, 1);

    grid
}
