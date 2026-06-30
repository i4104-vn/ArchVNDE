use gtk4::prelude::*;
use archvnde_common::desktop::DesktopApp;

pub fn create_app_button(app_item: &DesktopApp) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("switcher-item-btn");
    
    let app_icon_str = app_item.icon.as_deref().unwrap_or("application-x-executable");
    
    // Check if we have a cached screenshot for this app
    let mut screenshot_path = None;
    if let Some(ref app_id) = app_item.app_id {
        let path = format!("/tmp/archvnde-switcher-cache/{}.png", app_id);
        if std::path::Path::new(&path).exists() {
            screenshot_path = Some(path);
        }
    }
    
    if screenshot_path.is_none() {
        if !app_item.exec.is_empty() {
            let path = format!("/tmp/archvnde-switcher-cache/{}.png", app_item.exec);
            if std::path::Path::new(&path).exists() {
                screenshot_path = Some(path);
            }
        }
    }

    let preview_width = 240;
    let preview_height = 150;

    let base_widget: gtk4::Widget = if let Some(path) = screenshot_path {
        if let Ok(pb) = gdk_pixbuf::Pixbuf::from_file_at_scale(&path, preview_width, preview_height, false) {
            let texture = gdk4::Texture::for_pixbuf(&pb);
            let picture = gtk4::Picture::for_paintable(&texture);
            picture.set_size_request(preview_width, preview_height);
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.add_css_class("switcher-item-screenshot");
            picture.upcast()
        } else {
            create_placeholder_preview(app_icon_str, preview_width, preview_height)
        }
    } else {
        create_placeholder_preview(app_icon_str, preview_width, preview_height)
    };

    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(&base_widget));

    // 1. App Icon (top-left) - wrapped in a counter-skewed container
    let icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    icon_container.add_css_class("switcher-item-icon-container");
    icon_container.set_valign(gtk4::Align::Start);
    icon_container.set_halign(gtk4::Align::Start);
    icon_container.set_margin_top(8);
    icon_container.set_margin_start(8);

    let icon_widget = archvnde_common::icon::get_system_or_file_icon(app_icon_str, "application-x-executable");
    icon_widget.set_pixel_size(20);
    icon_widget.add_css_class("switcher-item-icon");
    icon_container.append(&icon_widget);
    overlay.add_overlay(&icon_container);

    // 2. Title Label (bottom-center) - matches bottom slanted line, text itself is counter-skewed
    let title_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    title_container.add_css_class("switcher-item-title-container");
    title_container.set_valign(gtk4::Align::End);
    title_container.set_halign(gtk4::Align::Fill);
    
    let title_label = gtk4::Label::new(Some(&app_item.name));
    title_label.add_css_class("switcher-app-title");
    title_label.set_halign(gtk4::Align::Center);
    title_label.set_hexpand(true);
    title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    title_label.set_max_width_chars(18);
    
    title_container.append(&title_label);
    overlay.add_overlay(&title_container);

    btn.set_child(Some(&overlay));
    
    btn
}


fn create_placeholder_preview(app_icon_str: &str, width: i32, height: i32) -> gtk4::Widget {
    let placeholder_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    placeholder_box.add_css_class("switcher-item-placeholder");
    placeholder_box.set_size_request(width, height);
    placeholder_box.set_valign(gtk4::Align::Center);
    placeholder_box.set_halign(gtk4::Align::Center);

    let icon_widget = archvnde_common::icon::get_system_or_file_icon(app_icon_str, "application-x-executable");
    icon_widget.set_pixel_size(48);
    icon_widget.add_css_class("switcher-item-icon");
    icon_widget.set_valign(gtk4::Align::Center);
    icon_widget.set_halign(gtk4::Align::Center);

    placeholder_box.append(&icon_widget);
    placeholder_box.upcast()
}

