use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{load_dock_config, DockConfig};
use super::rebuild_dock_content;

thread_local! {
    static DOCK_MONITOR: RefCell<Option<gio::FileMonitor>> = const { RefCell::new(None) };
}

pub fn start_dock_config_monitor(dock_box: gtk4::Box, config: Rc<RefCell<DockConfig>>) {
    let config_path = crate::config::get_dock_config_path();
    let file = gio::File::for_path(&config_path);
    if let Ok(monitor) = file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        let config_clone = config;
        let dock_box_clone = dock_box;
        monitor.connect_changed(move |_, _, _, event_type| {
            if event_type == gio::FileMonitorEvent::Changed || event_type == gio::FileMonitorEvent::Created {
                println!("dock.toml changed. Reloading configuration...");
                let new_config = crate::config::load_dock_config();
                *config_clone.borrow_mut() = new_config;
                rebuild_dock_content(&dock_box_clone, config_clone.clone());
            }
        });
        DOCK_MONITOR.with(|m| *m.borrow_mut() = Some(monitor));
    }
}
