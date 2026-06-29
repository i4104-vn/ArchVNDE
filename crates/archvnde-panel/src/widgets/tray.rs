use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates a horizontal Box container containing active system tray icons.
/// It polls the `archvnde-tray` registry every 2 seconds and reconstructs
/// buttons only if the list of registered services changes.
pub fn create_tray_widget() -> gtk4::Box {
    let tray_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    tray_container.add_css_class("panel-tray-box");
    tray_container.set_valign(gtk4::Align::Center);

    let displayed_services = Rc::new(RefCell::new(Vec::<String>::new()));

    let tray_container_clone = tray_container.clone();
    let displayed_clone = displayed_services.clone();

    // Poll D-Bus registered tray items every 2 seconds.
    // If services changed, rebuild the children widgets.
    gtk4::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        let current_items = archvnde_tray::get_tray_items();
        let mut current_services: Vec<String> = current_items.iter().map(|x| x.service.clone()).collect();
        current_services.sort();

        let mut prev_services = displayed_clone.borrow().clone();
        prev_services.sort();

        if current_services != prev_services {
            // Remove all existing widgets in the tray container
            while let Some(child) = tray_container_clone.first_child() {
                tray_container_clone.remove(&child);
            }

            // Populate fresh tray items
            for item in &current_items {
                let btn = gtk4::Button::new();
                btn.add_css_class("panel-tray-item-btn");
                btn.set_tooltip_text(Some(&item.title));
                btn.set_valign(gtk4::Align::Center);
                btn.set_halign(gtk4::Align::Center);

                let icon = archvnde_common::icon::get_system_or_file_icon(&item.icon_name, "image-missing");
                icon.set_pixel_size(16);
                btn.set_child(Some(&icon));

                let service_name = item.service.clone();
                btn.connect_clicked(move |_| {
                    println!("Activating tray icon service: {}", service_name);
                    archvnde_tray::activate_item(&service_name);
                });

                tray_container_clone.append(&btn);
            }

            // Sync the cached state
            *displayed_clone.borrow_mut() = current_items.iter().map(|x| x.service.clone()).collect();
        }

        gtk4::glib::ControlFlow::Continue
    });

    tray_container
}
