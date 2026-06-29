use gtk4::prelude::*;
use archvnde_common::desktop::DesktopApp;

pub fn create_app_button(app_item: &DesktopApp) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("switcher-item-btn");
    
    let btn_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    btn_box.set_valign(gtk4::Align::Center);
    btn_box.set_halign(gtk4::Align::Center);

    let app_icon_str = app_item.icon.as_deref().unwrap_or("application-x-executable");
    let icon_widget = archvnde_common::icon::get_system_or_file_icon(app_icon_str, "application-x-executable");
    icon_widget.set_pixel_size(36);
    icon_widget.add_css_class("switcher-item-icon");
    icon_widget.set_valign(gtk4::Align::Center);
    icon_widget.set_halign(gtk4::Align::Center);

    btn_box.append(&icon_widget);
    btn.set_child(Some(&btn_box));
    
    btn
}
