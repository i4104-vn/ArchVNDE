use crate::models::DesktopApp;
use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use super::app_row::create_grid_app_widget;
use archvnde_common::models::DockConfig;

pub fn populate_grid(
    scrolled_window: &gtk4::ScrolledWindow,
    window: &gtk4::ApplicationWindow,
    apps: &[DesktopApp],
    query: &str,
    config: Rc<RefCell<DockConfig>>,
    populate_grid_ref: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
) {
    let query_lower = query.to_lowercase();
    let filtered: Vec<DesktopApp> = apps
        .iter()
        .filter(|app| app.name.to_lowercase().contains(&query_lower))
        .cloned()
        .collect();

    if filtered.is_empty() {
        let no_results = gtk4::Label::new(Some("No applications found"));
        no_results.add_css_class("launcher-no-results");
        no_results.set_halign(gtk4::Align::Center);
        no_results.set_valign(gtk4::Align::Center);
        no_results.set_hexpand(true);
        no_results.set_vexpand(true);
        scrolled_window.set_child(Some(&no_results));
    } else {
        let grid = gtk4::Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(15);
        grid.set_column_homogeneous(true);
        grid.add_css_class("launcher-grid");

        let reload_cb = {
            let populate_grid_ref = populate_grid_ref.clone();
            Rc::new(move || {
                if let Some(ref f) = *populate_grid_ref.borrow() {
                    f();
                }
            }) as Rc<dyn Fn()>
        };

        for (i, app) in filtered.into_iter().enumerate() {
            let col = (i % 4) as i32;
            let row = (i / 4) as i32;
            let app_btn = create_grid_app_widget(&app, window, config.clone(), reload_cb.clone());
            grid.attach(&app_btn, col, row, 1, 1);
        }
        scrolled_window.set_child(Some(&grid));
    }
}
