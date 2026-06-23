use gtk4::prelude::*;

pub fn create_power_actions_row() -> gtk4::Box {
    let power_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    power_box.set_margin_top(6);
    power_box.set_homogeneous(true);

    let power_off = gtk4::Button::with_label("⏻  Power Off");
    power_off.add_css_class("power-btn");
    power_off.connect_clicked(move |_| {
        println!("Power Off requested...");
        let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
    });

    let logout = gtk4::Button::with_label("↪  Log Out");
    logout.add_css_class("power-btn");
    logout.add_css_class("logout-btn");
    logout.connect_clicked(move |_| {
        println!("Log Out requested...");
        if let Ok(user) = std::env::var("USER") {
            let _ = std::process::Command::new("loginctl").args(["terminate-user", &user]).spawn();
        }
    });

    power_box.append(&power_off);
    power_box.append(&logout);
    power_box
}
