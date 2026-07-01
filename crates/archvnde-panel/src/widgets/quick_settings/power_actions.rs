use gtk4::prelude::*;

pub fn create_header_row() -> gtk4::Box {
    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    header_box.set_hexpand(true);

    // Left: Title
    let title = gtk4::Label::new(Some("Quick Settings"));
    title.add_css_class("quick-settings-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);

    // Right: Action buttons container
    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk4::Align::End);

    // 1. Settings button
    let settings_btn = gtk4::Button::new();
    settings_btn.add_css_class("circle-btn");
    let settings_icon = archvnde_icon::get_icon("settings", 16);
    settings_btn.set_child(Some(&settings_icon));
    settings_btn.connect_clicked(|_| {
        println!("Settings window triggered...");
    });

    // 2. Power off button
    let power_off = gtk4::Button::new();
    power_off.add_css_class("circle-btn");
    power_off.add_css_class("power-btn");
    let power_icon = archvnde_icon::get_icon("power", 16);
    power_off.set_child(Some(&power_icon));
    power_off.connect_clicked(|_| {
        println!("Power Off requested...");
        let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
    });

    btn_box.append(&settings_btn);
    btn_box.append(&power_off);

    header_box.append(&title);
    header_box.append(&btn_box);

    header_box
}
