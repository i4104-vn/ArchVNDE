use gtk4::prelude::*;

pub fn build_empty_state(main_box: &gtk4::Box, window: &gtk4::ApplicationWindow) {
    main_box.set_spacing(16);
    main_box.set_margin_top(30);
    main_box.set_margin_bottom(30);
    main_box.set_margin_start(50);
    main_box.set_margin_end(50);
    main_box.set_halign(gtk4::Align::Center);
    main_box.set_valign(gtk4::Align::Center);

    let no_apps_icon = archvnde_common::icon::get_system_or_file_icon("application-x-executable", "application-x-executable");
    no_apps_icon.set_pixel_size(48);
    no_apps_icon.set_halign(gtk4::Align::Center);

    let no_apps_lbl = gtk4::Label::new(Some("Không có ứng dụng nào đang chạy"));
    no_apps_lbl.add_css_class("switcher-app-title");
    no_apps_lbl.set_halign(gtk4::Align::Center);

    main_box.append(&no_apps_icon);
    main_box.append(&no_apps_lbl);

    window.set_child(Some(main_box));

    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    
    let window_close = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        match key {
            gtk4::gdk::Key::Escape | gtk4::gdk::Key::Return => {
                window_close.close();
                gtk4::glib::Propagation::Stop
            }
            _ => gtk4::glib::Propagation::Proceed,
        }
    });

    let window_release = window.clone();
    key_controller.connect_key_released(move |_, key, _, _| {
        match key {
            gtk4::gdk::Key::Alt_L | gtk4::gdk::Key::Alt_R => {
                window_release.close();
            }
            _ => {}
        }
    });

    window.add_controller(key_controller);
    window.present();
}
