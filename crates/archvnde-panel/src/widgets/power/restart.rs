use gtk4::prelude::*;
use std::process::Command;

/// Triggers a system reboot.
pub fn trigger_restart() {
    println!("Reboot requested...");
    let _ = Command::new("systemctl").arg("reboot").spawn();
}

/// Creates a styled GTK Button that triggers a system reboot when clicked.
pub fn create_restart_button() -> gtk4::Button {
    let restart_btn = gtk4::Button::new();
    restart_btn.add_css_class("circle-btn");
    restart_btn.add_css_class("restart-btn");
    let restart_icon = archvnde_icon::get_icon("performance", 16);
    restart_btn.set_child(Some(&restart_icon));
    restart_btn.connect_clicked(|_| {
        trigger_restart();
    });
    restart_btn
}
