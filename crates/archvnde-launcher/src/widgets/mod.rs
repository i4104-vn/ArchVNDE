pub mod app_row;
pub mod grid;
pub mod footer;

use crate::core::find_desktop_apps;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::rc::Rc;
use std::cell::RefCell;
use archvnde_common::config::load_dock_config;
use grid::populate_grid;
use footer::create_launcher_footer;

pub fn build_launcher_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    
    // Use the unified layout helper from archvnde_common
    archvnde_common::window::init_layer_window(
        &window,
        Layer::Overlay,
        KeyboardMode::Exclusive,
        -1, // no exclusive zone needed for overlay launcher popups
        &[
            (Edge::Top, false),
            (Edge::Bottom, true),
            (Edge::Left, false),
            (Edge::Right, false),
        ],
        84, // margin bottom
    );

    window.set_default_size(450, 550);
    window.add_css_class("launcher-window");

    let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    box_layout.add_css_class("launcher-box");
    box_layout.set_margin_start(16);
    box_layout.set_margin_end(16);
    box_layout.set_margin_top(16);
    box_layout.set_margin_bottom(16);

    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some("Search apps, settings..."));
    search_entry.add_css_class("launcher-search");

    let scrolled_window = gtk4::ScrolledWindow::new();
    scrolled_window.set_vexpand(true);

    let config = Rc::new(RefCell::new(load_dock_config()));
    let apps = find_desktop_apps();
    let apps_rc = Rc::new(apps);

    let current_query = Rc::new(RefCell::new(String::new()));
    let populate_grid_ref = Rc::new(RefCell::new(None));
    let populate_grid_ref_c = populate_grid_ref.clone();

    let populate_impl = {
        let current_query = current_query.clone();
        let apps_rc = apps_rc.clone();
        let scrolled_window = scrolled_window.clone();
        let window = window.clone();
        let config = config.clone();
        let populate_grid_ref = populate_grid_ref.clone();

        move || {
            populate_grid(
                &scrolled_window,
                &window,
                &apps_rc,
                &current_query.borrow(),
                config.clone(),
                populate_grid_ref.clone(),
            );
        }
    };

    *populate_grid_ref_c.borrow_mut() = Some(Rc::new(populate_impl) as Rc<dyn Fn()>);

    // Initial populate
    if let Some(ref f) = *populate_grid_ref.borrow() {
        f();
    }

    let current_query_search = current_query.clone();
    let populate_grid_search = populate_grid_ref.clone();
    search_entry.connect_changed(move |entry| {
        *current_query_search.borrow_mut() = entry.text().to_string();
        if let Some(ref f) = *populate_grid_search.borrow() {
            f();
        }
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

    box_layout.append(&search_entry);
    box_layout.append(&scrolled_window);

    // Separator above footer
    let footer_sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    footer_sep.add_css_class("launcher-footer-separator");
    box_layout.append(&footer_sep);

    // Footer section
    let footer = create_launcher_footer();
    box_layout.append(&footer);

    window.set_child(Some(&box_layout));

    archvnde_common::animation::genie_in(box_layout.upcast_ref(), 450, 550, 240);

    window
}
