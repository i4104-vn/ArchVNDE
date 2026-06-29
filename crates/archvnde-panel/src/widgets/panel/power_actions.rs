use gtk4::prelude::*;

pub fn create_header_row() -> gtk4::Box {
    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    header_box.set_hexpand(true);

    // Left: Title
    let title = gtk4::Label::new(Some(&archvnde_common::i18n::t("control.title")));
    title.add_css_class("control-center-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);

    // Right: Action buttons container
    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk4::Align::End);

    // 1. Language Toggle Button
    let lang_btn = gtk4::Button::new();
    lang_btn.add_css_class("circle-btn");
    lang_btn.set_tooltip_text(Some("Switch Language / Đổi ngôn ngữ"));
    
    let current_lang = archvnde_common::i18n::get_locale().to_uppercase();
    let lang_lbl = gtk4::Label::new(Some(&current_lang));
    lang_lbl.add_css_class("control-lang-label");
    lang_btn.set_child(Some(&lang_lbl));

    let lang_lbl_clone = lang_lbl.clone();
    lang_btn.connect_clicked(move |_| {
        let new_locale = if archvnde_common::i18n::get_locale() == "vi" { "en" } else { "vi" };
        archvnde_common::i18n::set_locale(new_locale);
        lang_lbl_clone.set_text(&new_locale.to_uppercase());

        // Send a system notification alerting the user to restart widgets for full changes
        let (notif_title, notif_msg) = if new_locale == "en" {
            ("Language Changed", "Restart widgets to apply changes system-wide.")
        } else {
            ("Đã thay đổi ngôn ngữ", "Khởi động lại widgets để áp dụng toàn hệ thống.")
        };

        let _ = std::process::Command::new("notify-send")
            .args(&["-i", "preferences-desktop-locale", notif_title, notif_msg])
            .spawn();
    });

    // 2. Settings button
    let settings_btn = gtk4::Button::new();
    settings_btn.add_css_class("circle-btn");
    let settings_icon = archvnde_common::icon::get_icon("settings", 16);
    settings_btn.set_child(Some(&settings_icon));
    settings_btn.connect_clicked(|_| {
        println!("Settings window triggered...");
    });

    // 3. Power off button
    let power_off = crate::widgets::power::create_shutdown_button();

    btn_box.append(&lang_btn);
    btn_box.append(&settings_btn);
    btn_box.append(&power_off);

    header_box.append(&title);
    header_box.append(&btn_box);

    header_box
}
