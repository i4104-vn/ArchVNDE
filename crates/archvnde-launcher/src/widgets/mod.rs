pub mod app_row;

use crate::core::find_desktop_apps;
use crate::models::DesktopApp;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::rc::Rc;
use app_row::create_app_row;

pub fn build_launcher_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_default_size(450, 550);
    window.add_css_class("launcher-window");

    let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 15);
    box_layout.add_css_class("launcher-box");
    box_layout.set_margin_start(20);
    box_layout.set_margin_end(20);
    box_layout.set_margin_top(20);
    box_layout.set_margin_bottom(20);

    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some("Search apps, files, settings..."));
    search_entry.add_css_class("launcher-search");

    let scrolled_window = gtk4::ScrolledWindow::new();
    scrolled_window.set_vexpand(true);

    let list_box = gtk4::ListBox::new();
    list_box.add_css_class("launcher-list");

    let apps = find_desktop_apps();
    let apps_rc = Rc::new(apps);

    let populate_list = {
        let list_box = list_box.clone();
        let window_clone = window.clone();
        move |filtered_apps: Vec<DesktopApp>| {
            while let Some(row) = list_box.first_child() {
                list_box.remove(&row);
            }
            for app in filtered_apps {
                let row = create_app_row(&app, &window_clone);
                list_box.append(&row);
            }
        }
    };

    populate_list(apps_rc.as_ref().clone());

    let apps_search = apps_rc.clone();
    let populate_search = populate_list.clone();
    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_string().to_lowercase();
        let filtered: Vec<DesktopApp> = apps_search
            .iter()
            .filter(|app| app.name.to_lowercase().contains(&query))
            .cloned()
            .collect();
        populate_search(filtered);
    });

    let is_animating = Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();
    let win_clone_close = window.clone();
    let box_layout_clone_close = box_layout.clone();
    window.connect_close_request(move |_| {
        if is_animating_clone.get() {
            return gtk4::glib::Propagation::Proceed;
        }
        is_animating_clone.set(true);
        let win_cb = win_clone_close.clone();
        let box_layout_cb = box_layout_clone_close.clone();
        let w = box_layout_cb.width().max(450);
        let h = box_layout_cb.height().max(550);
        archvnde_common::animation::genie_out(
            box_layout_cb.upcast_ref(),
            w,
            h,
            200,
            move || {
                win_cb.destroy();
            }
        );
        gtk4::glib::Propagation::Stop
    });

    window.connect_is_active_notify(|win| {
        if !win.is_active() {
            win.close();
        }
    });

    let key_controller = gtk4::EventControllerKey::new();
    let win_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            win_clone.close();
            gtk4::glib::Propagation::Proceed
        } else {
            gtk4::glib::Propagation::Stop
        }
    });
    window.add_controller(key_controller);

    scrolled_window.set_child(Some(&list_box));
    box_layout.append(&search_entry);
    box_layout.append(&scrolled_window);
    window.set_child(Some(&box_layout));

    archvnde_common::animation::genie_in(box_layout.upcast_ref(), 450, 550, 240);

    window
}
