use gtk4::prelude::*;
use std::process::Command;

/// Triggers a system poweroff.
pub fn trigger_shutdown() {
    println!("Power Off requested...");
    let _ = Command::new("systemctl").arg("poweroff").spawn();
}

/// Creates a styled GTK Button that triggers a system shutdown when clicked.
pub fn create_shutdown_button() -> gtk4::Button {
    let power_off = gtk4::Button::new();
    power_off.add_css_class("circle-btn");
    power_off.add_css_class("power-btn");
    let power_icon = archvnde_icon::get_icon("power", 16);
    power_off.set_child(Some(&power_icon));
    power_off.connect_clicked(|_| {
        trigger_shutdown();
    });
    power_off
}
