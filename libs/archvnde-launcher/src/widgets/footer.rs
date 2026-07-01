//! UI layout renderer for the launcher footer row.

use gtk4::prelude::*;

<<<<<<< HEAD:libs/archvnde-launcher/src/widgets/footer.rs
pub fn create_launcher_footer() -> gtk4::Box {
=======
/// Builds a horizontal bar containing user profile details, spacing, and a power popover trigger.
pub fn build_footer_layout() -> (
    gtk4::Box,
    gtk4::Button,
    gtk4::Popover,
    gtk4::Button,
    gtk4::Button,
    gtk4::Button,
) {
>>>>>>> 52145a1 (refactor: clean up comments and add i18n support):libs/archvnde-launcher/src/widgets/footer/render.rs
    let footer_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    footer_box.add_css_class("launcher-footer-box");

    let profile_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    profile_box.set_valign(gtk4::Align::Center);
    let user_icon = archvnde_common::icon::get_icon_colored("user", 20, "#ffffff");
    let username = std::env::var("USER").unwrap_or_else(|_| "User".to_string());
    let user_label = gtk4::Label::new(Some(&username));
    user_label.add_css_class("launcher-profile-label");
    profile_box.append(&user_icon);
    profile_box.append(&user_label);

    let power_btn = gtk4::Button::new();
    power_btn.add_css_class("launcher-power-btn");
    let power_icon = archvnde_common::icon::get_icon_colored("power", 20, "#ff5555");
    power_btn.set_child(Some(&power_icon));

    let power_popover = gtk4::Popover::new();
    power_popover.set_parent(&power_btn);
    power_popover.set_has_arrow(true);

    let power_menu = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    power_menu.add_css_class("dock-menu-box");

<<<<<<< HEAD:libs/archvnde-launcher/src/widgets/footer.rs
    let shutdown_btn = gtk4::Button::with_label("Shut Down");
    shutdown_btn.add_css_class("menu-item-btn");
    shutdown_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
    });

    let reboot_btn = gtk4::Button::with_label("Restart");
    reboot_btn.add_css_class("menu-item-btn");
    reboot_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("systemctl").arg("reboot").spawn();
    });

    let suspend_btn = gtk4::Button::with_label("Suspend");
    suspend_btn.add_css_class("menu-item-btn");
    suspend_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("systemctl").arg("suspend").spawn();
    });
=======
    let shutdown_btn = gtk4::Button::with_label(&archvnde_common::i18n::t("launcher.shutdown"));
    shutdown_btn.add_css_class("launcher-menu-item-btn");

    let reboot_btn = gtk4::Button::with_label(&archvnde_common::i18n::t("launcher.restart"));
    reboot_btn.add_css_class("launcher-menu-item-btn");

    let suspend_btn = gtk4::Button::with_label(&archvnde_common::i18n::t("launcher.suspend"));
    suspend_btn.add_css_class("launcher-menu-item-btn");
>>>>>>> 52145a1 (refactor: clean up comments and add i18n support):libs/archvnde-launcher/src/widgets/footer/render.rs

    power_menu.append(&shutdown_btn);
    power_menu.append(&reboot_btn);
    power_menu.append(&suspend_btn);
    power_popover.set_child(Some(&power_menu));

    let power_popover_clone = power_popover.clone();
    power_btn.connect_clicked(move |_| {
        power_popover_clone.popup();
    });

    footer_box.append(&profile_box);
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);
    footer_box.append(&power_btn);

    footer_box
}

