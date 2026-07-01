pub mod item;
pub mod popovers;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{load_dock_config, DockConfig};
use item::create_dock_button;
use popovers::{attach_unpin_popover, create_pin_app_button};

pub fn rebuild_dock_content(dock_box: &gtk4::Box, config: Rc<RefCell<DockConfig>>) {
    // Clear all existing widgets
    while let Some(child) = dock_box.first_child() {
        dock_box.remove(&child);
    }

    // 1. App Launcher (logo icon) - fixed
    let launcher_btn = create_dock_button("logo", "Application Launcher", "archvnde-launcher", &[]);
    dock_box.append(&launcher_btn);

    // Separator
    let sep1 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep1.add_css_class("dock-separator");
    dock_box.append(&sep1);

    // 2. Render pinned apps from configuration
    let pinned_apps = config.borrow().pinned_apps.clone();
    for app in pinned_apps {
        let args_ref: Vec<&str> = app.args.iter().map(|s| s.as_str()).collect();
        let btn = create_dock_button(&app.icon, &app.name, &app.command, &args_ref);
        
        attach_unpin_popover(&btn, app, config.clone(), dock_box.clone());
        dock_box.append(&btn);
    }

    // Separator
    let sep2 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep2.add_css_class("dock-separator");
    dock_box.append(&sep2);

    // 3. Pin App button (+)
    let pin_btn = create_pin_app_button(config.clone(), dock_box.clone());
    dock_box.append(&pin_btn);

    // Separator
    let sep3 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep3.add_css_class("dock-separator");
    dock_box.append(&sep3);

    // 4. Trash Bin - fixed
    let trash_btn = create_dock_button("trash", "Trash Bin", "pcmanfm", &["trash:///"]);
    dock_box.append(&trash_btn);
}

pub fn build_dock_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();

    // Assign to Top layer so it floats above normal windows
    window.set_layer(Layer::Top);

    // Set an exclusive zone of 64px so other windows respect the dock space at the bottom
    window.set_exclusive_zone(64);

    // Anchor ONLY to the bottom so it dynamically centers horizontally!
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Top, false);

    // Add margin to make it float above the bottom screen edge elegantly (macOS style)
    window.set_margin(Edge::Bottom, 10);

    window.add_css_class("dock-window");

    let dock_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    dock_box.add_css_class("dock-box");

    let config = Rc::new(RefCell::new(load_dock_config()));
    rebuild_dock_content(&dock_box, config);

    window.set_child(Some(&dock_box));

    window
}
