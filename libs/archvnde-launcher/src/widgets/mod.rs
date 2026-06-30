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
    box_layout.set_margin_top(50);
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

    let mut app_widgets = Vec::new();
    for app in apps_rc.iter() {
        let btn = create_list_app_widget(app, &window);
        left_list_box.append(&btn);
        app_widgets.push((app.clone(), btn));
    }
    let app_widgets_rc = Rc::new(app_widgets);
    left_scroll.set_child(Some(&left_list_box));

    // Right column: dynamic search results
    let right_scroll = gtk4::ScrolledWindow::new();
    right_scroll.add_css_class("launcher-right-column");
    right_scroll.set_hexpand(true);
    right_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

    let current_query = Rc::new(RefCell::new(String::new()));
    let selected_index = Rc::new(RefCell::new(Some(0)));

    let populate_impl = {
        let current_query = current_query.clone();
        let app_widgets = app_widgets_rc.clone();
        let right_scroll = right_scroll.clone();
        let window = window.clone();

        move || {
            let query = current_query.borrow().trim().to_lowercase();

            // 1. Filter applications on the left column in real-time by toggling visibility
            for (app, btn) in app_widgets.iter() {
                let matches = query.is_empty() || app.name.to_lowercase().contains(&query);
                btn.set_visible(matches);
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
    update_highlight(&left_list_box, &right_scroll, Some(0));

    let debounce_source_id: Rc<RefCell<Option<gtk4::glib::SourceId>>> = Rc::new(RefCell::new(None));
    let current_query_search = current_query.clone();
    let populate_grid_search = populate_impl_rc.clone();
    let d_source_id = debounce_source_id.clone();
    let selected_index_changed = selected_index.clone();
    let left_list_box_changed = left_list_box.clone();
    let right_scroll_changed = right_scroll.clone();

    search_entry.connect_changed(move |entry| {
        *current_query_search.borrow_mut() = entry.text().to_string();
        
        // Debounce input to reduce main thread lag from directory searches
        if let Some(source_id) = d_source_id.borrow_mut().take() {
            source_id.remove();
        }
        
        let populate_clone = populate_grid_search.clone();
        let d_source_id_clone = d_source_id.clone();
        let selected_index_clone = selected_index_changed.clone();
        let left_list_box_clone = left_list_box_changed.clone();
        let right_scroll_clone = right_scroll_changed.clone();

        let new_source_id = gtk4::glib::timeout_add_local_once(
            std::time::Duration::from_millis(150),
            move || {
                populate_clone();
                *d_source_id_clone.borrow_mut() = None;
                *selected_index_clone.borrow_mut() = Some(0);
                update_highlight(&left_list_box_clone, &right_scroll_clone, Some(0));
            }
        );
        *d_source_id.borrow_mut() = Some(new_source_id);
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
    let selected_index_clone = selected_index.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            win_clone.close();
            gtk4::glib::Propagation::Proceed
        } else if key == gtk4::gdk::Key::Down {
            let buttons = get_visible_selectable_buttons(&left_list_box_clone, &right_scroll_clone);
            if !buttons.is_empty() {
                let mut current = selected_index_clone.borrow().unwrap_or(0);
                if selected_index_clone.borrow().is_none() {
                    current = 0;
                } else {
                    current = (current + 1) % buttons.len();
                }
                *selected_index_clone.borrow_mut() = Some(current);
                update_highlight(&left_list_box_clone, &right_scroll_clone, Some(current));
            }
            gtk4::glib::Propagation::Stop
        } else if key == gtk4::gdk::Key::Up {
            let buttons = get_visible_selectable_buttons(&left_list_box_clone, &right_scroll_clone);
            if !buttons.is_empty() {
                let mut current = selected_index_clone.borrow().unwrap_or(0);
                if selected_index_clone.borrow().is_none() {
                    current = buttons.len() - 1;
                } else {
                    if current == 0 {
                        current = buttons.len() - 1;
                    } else {
                        current -= 1;
                    }
                }
                *selected_index_clone.borrow_mut() = Some(current);
                update_highlight(&left_list_box_clone, &right_scroll_clone, Some(current));
            }
            gtk4::glib::Propagation::Stop
        } else if key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter {
            let buttons = get_visible_selectable_buttons(&left_list_box_clone, &right_scroll_clone);
            if let Some(idx) = *selected_index_clone.borrow() {
                if idx < buttons.len() {
                    buttons[idx].activate();
                }
            } else if !buttons.is_empty() {
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

fn update_highlight(
    left_list_box: &gtk4::Box,
    right_scroll: &gtk4::ScrolledWindow,
    selected_idx: Option<usize>,
) {
    let buttons = get_visible_selectable_buttons(left_list_box, right_scroll);
    for (i, btn) in buttons.iter().enumerate() {
        if Some(i) == selected_idx {
            btn.add_css_class("selected");
        } else {
            btn.remove_css_class("selected");
        }
    }
}
