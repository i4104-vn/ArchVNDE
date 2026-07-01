pub mod app_row;
pub mod footer;
pub mod file_search;
pub mod search;

use crate::core::find_desktop_apps;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::rc::Rc;
use std::cell::RefCell;
use footer::create_launcher_footer;
use app_row::create_list_app_widget;
use search::populate_search_results;

pub fn build_launcher_ui(
    app: &gtk4::Application,
    launcher_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    
    archvnde_common::window::init_layer_window(
        &window,
        Layer::Overlay,
        KeyboardMode::OnDemand,
        -1,
        &[
            (Edge::Top, true),
            (Edge::Bottom, true),
            (Edge::Left, true),
            (Edge::Right, true),
        ],
        -1,
    );

    window.add_css_class("launcher-window");

    let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    box_layout.add_css_class("launcher-box");
    box_layout.set_halign(gtk4::Align::Start);
    box_layout.set_valign(gtk4::Align::Start);
    box_layout.set_size_request(780, 560);
    box_layout.set_margin_top(6);
    box_layout.set_margin_start(12);

    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some(&archvnde_common::i18n::t("launcher.search_placeholder")));
    search_entry.add_css_class("launcher-search");
    search_entry.set_margin_top(16);
    search_entry.set_margin_start(16);
    search_entry.set_margin_end(16);

    // Horizontal split box for two columns
    let columns_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    columns_box.add_css_class("launcher-columns-box");
    columns_box.set_vexpand(true);

    // Left column: scrollable all apps list (always shows all apps)
    let left_scroll = gtk4::ScrolledWindow::new();
    left_scroll.add_css_class("launcher-left-column");
    left_scroll.set_size_request(280, -1);
    left_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

    let left_list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let apps = find_desktop_apps();
    let apps_rc = Rc::new(apps);

    for app in apps_rc.iter() {
        let btn = create_list_app_widget(app, &window);
        left_list_box.append(&btn);
    }
    left_scroll.set_child(Some(&left_list_box));

    // Right column: dynamic search results
    let right_scroll = gtk4::ScrolledWindow::new();
    right_scroll.add_css_class("launcher-right-column");
    right_scroll.set_hexpand(true);
    right_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

    let current_query = Rc::new(RefCell::new(String::new()));

    let populate_impl = {
        let current_query = current_query.clone();
        let apps_rc = apps_rc.clone();
        let right_scroll = right_scroll.clone();
        let left_list_box = left_list_box.clone();
        let window = window.clone();

        move || {
            let query = current_query.borrow().trim().to_lowercase();

            // 1. Filter applications on the left column in real-time
            while let Some(child) = left_list_box.first_child() {
                left_list_box.remove(&child);
            }
            for app in apps_rc.iter() {
                if query.is_empty() || app.name.to_lowercase().contains(&query) {
                    let btn = create_list_app_widget(app, &window);
                    left_list_box.append(&btn);
                }
            }

            // 2. Populate file search and web search on the right column
            populate_search_results(
                &right_scroll,
                &current_query.borrow(),
                &window,
            );
        }
    };

    let populate_impl_rc = Rc::new(populate_impl);

    // Initial populate
    populate_impl_rc();

    let current_query_search = current_query.clone();
    let populate_grid_search = populate_impl_rc.clone();
    search_entry.connect_changed(move |entry| {
        *current_query_search.borrow_mut() = entry.text().to_string();
        populate_grid_search();
    });

    let is_animating = Rc::new(std::cell::Cell::new(false));
    let is_animating_clone = is_animating.clone();
    let win_clone_close = window.clone();
    let box_layout_clone_close = box_layout.clone();
    let lw_inner = launcher_window.clone();
    window.connect_close_request(move |_| {
        if is_animating_clone.get() {
            return gtk4::glib::Propagation::Stop;
        }
        is_animating_clone.set(true);
        if let Ok(mut borrow) = lw_inner.try_borrow_mut() {
            *borrow = None;
        }
        let win_cb = win_clone_close.clone();
        let box_layout_cb = box_layout_clone_close.clone();
        archvnde_common::animation::slide_out_cb(
            box_layout_cb.upcast_ref(),
            archvnde_common::animation::SlideDirection::Up,
            40,
            450,
            false,
            move || {
                win_cb.destroy();
            }
        );
        gtk4::glib::Propagation::Stop
    });

    // Dismiss when clicking outside the launcher box area
    let click_gesture = gtk4::GestureClick::new();
    let box_layout_c = box_layout.clone();
    let window_c = window.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let picked = window_c.pick(x, y, gtk4::PickFlags::DEFAULT);
        let inside = picked
            .map(|w| w.is_ancestor(&box_layout_c) || w == box_layout_c)
            .unwrap_or(false);
        if !inside {
            window_c.close();
        }
    });
    window.add_controller(click_gesture);

    window.connect_is_active_notify(|win| {
        if !win.is_active() {
            win.close();
        }
    });

    let key_controller = gtk4::EventControllerKey::new();
    let win_clone = window.clone();
    let left_list_box_clone = left_list_box.clone();
    let right_scroll_clone = right_scroll.clone();
    let search_entry_clone = search_entry.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            win_clone.close();
            gtk4::glib::Propagation::Proceed
        } else if key == gtk4::gdk::Key::Down {
            let buttons = get_visible_selectable_buttons(&left_list_box_clone, &right_scroll_clone);
            if !buttons.is_empty() {
                let focused_w = gtk4::prelude::RootExt::focus(&win_clone);
                let current_index = focused_w.and_then(|fw| {
                    buttons.iter().position(|b| fw == b.clone().upcast::<gtk4::Widget>())
                });

                match current_index {
                    Some(idx) => {
                        let next_idx = (idx + 1) % buttons.len();
                        buttons[next_idx].grab_focus();
                    }
                    None => {
                        buttons[0].grab_focus();
                    }
                }
            }
            gtk4::glib::Propagation::Stop
        } else if key == gtk4::gdk::Key::Up {
            let buttons = get_visible_selectable_buttons(&left_list_box_clone, &right_scroll_clone);
            if !buttons.is_empty() {
                let focused_w = gtk4::prelude::RootExt::focus(&win_clone);
                let current_index = focused_w.and_then(|fw| {
                    buttons.iter().position(|b| fw == b.clone().upcast::<gtk4::Widget>())
                });

                match current_index {
                    Some(idx) => {
                        if idx == 0 {
                            search_entry_clone.grab_focus();
                        } else {
                            buttons[idx - 1].grab_focus();
                        }
                    }
                    None => {
                        buttons[buttons.len() - 1].grab_focus();
                    }
                }
            }
            gtk4::glib::Propagation::Stop
        } else if key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter {
            let focused_w = gtk4::prelude::RootExt::focus(&win_clone);
            let buttons = get_visible_selectable_buttons(&left_list_box_clone, &right_scroll_clone);

            if let Some(fw) = focused_w {
                if let Some(btn) = fw.downcast_ref::<gtk4::Button>() {
                    btn.activate();
                    return gtk4::glib::Propagation::Stop;
                }
            }

            if !buttons.is_empty() {
                buttons[0].activate();
            }
            gtk4::glib::Propagation::Stop
        } else {
            gtk4::glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    box_layout.append(&search_entry);
    
    columns_box.append(&left_scroll);
    columns_box.append(&right_scroll);
    columns_box.set_margin_start(16);
    columns_box.set_margin_end(16);
    box_layout.append(&columns_box);

    let footer_sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    footer_sep.add_css_class("launcher-footer-separator");
    footer_sep.set_margin_start(16);
    footer_sep.set_margin_end(16);
    box_layout.append(&footer_sep);

    let footer = create_launcher_footer();
    footer.set_margin_start(16);
    footer.set_margin_end(16);
    footer.set_margin_bottom(16);
    box_layout.append(&footer);

    window.set_child(Some(&box_layout));

    archvnde_common::animation::slide_in(
        box_layout.upcast_ref(),
        archvnde_common::animation::SlideDirection::Down,
        40,
        450,
    );

    search_entry.grab_focus();

    window
}

fn get_visible_selectable_buttons(
    left_list_box: &gtk4::Box,
    right_scroll: &gtk4::ScrolledWindow,
) -> Vec<gtk4::Button> {
    let mut buttons = Vec::new();

    // 1. Collect from left list box (all apps)
    let mut child = left_list_box.first_child();
    while let Some(w) = child {
        if w.is_visible() {
            if let Some(btn) = w.downcast_ref::<gtk4::Button>() {
                buttons.push(btn.clone());
            }
        }
        child = w.next_sibling();
    }

    // 2. Collect from right column
    if let Some(right_child) = right_scroll.child() {
        if let Some(right_box) = right_child.downcast_ref::<gtk4::Box>() {
            let mut sub_child = right_box.first_child();
            while let Some(w) = sub_child {
                if w.is_visible() {
                    if let Some(btn) = w.downcast_ref::<gtk4::Button>() {
                        buttons.push(btn.clone());
                    } else if let Some(files_box) = w.downcast_ref::<gtk4::Box>() {
                        let mut file_child = files_box.first_child();
                        while let Some(fw) = file_child {
                            if fw.is_visible() {
                                if let Some(btn) = fw.downcast_ref::<gtk4::Button>() {
                                    buttons.push(btn.clone());
                                }
                            }
                            file_child = fw.next_sibling();
                        }
                    }
                }
                sub_child = w.next_sibling();
            }
        }
    }

    buttons
}
