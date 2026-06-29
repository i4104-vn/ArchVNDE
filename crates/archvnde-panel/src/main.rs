mod widgets;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use widgets::panel::create_status_indicators;
use widgets::workspace::create_workspace_switcher;
use widgets::sys_monitor::create_sys_monitor_widget;
use widgets::tray::create_tray_widget;
use archvnde_island::create_system_island;

fn main() {
    println!("Starting ArchVNDE Panel...");

    // Initialize D-Bus StatusNotifierWatcher system tray listener daemon
    archvnde_tray::spawn_watcher_service();

    let application = gtk4::Application::new(
        Some("org.archvnde.panel"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);

        // Define shared window states for mutual exclusivity
        let control_center_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));
        let calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));
        let launcher_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Assign to the Top layer so it renders above normal windows
        window.set_layer(Layer::Top);

        // Set exclusive zone so other maximized windows don't overlap it
        window.set_exclusive_zone(44);

        // Anchor it to the top, left, and right edges of the screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Float the bar 6px from the top edge
        window.set_margin(Edge::Top, 6);
        window.set_margin(Edge::Left, 8);
        window.set_margin(Edge::Right, 8);

        // Set default height of the panel
        window.set_default_size(0, 36);

        // Add styling class
        window.add_css_class("panel-window");

        // Layout container
        let box_layout = gtk4::CenterBox::new();
        box_layout.add_css_class("panel-box");

        // 1. Logo Button (launches launcher)
        let logo_btn = gtk4::Button::new();
        logo_btn.add_css_class("panel-logo-btn");
        let logo_icon = archvnde_common::icon::get_icon("logo", 16);
        logo_btn.set_child(Some(&logo_icon));
        
        let lw_clone = launcher_window.clone();
        let ccw_clone = control_center_window.clone();
        let cw_clone = calendar_window.clone();
        let app_clone = app.clone();
        logo_btn.connect_clicked(move |_| {
            // Close other windows safely by releasing RefCell borrows first
            let cc_win = { ccw_clone.borrow().clone() };
            if let Some(win) = cc_win {
                win.close();
            }

            let cal_win = { cw_clone.borrow().clone() };
            if let Some(win) = cal_win {
                win.close();
            }
            
            let existing = { lw_clone.borrow().clone() };
            if let Some(win) = existing {
                win.close();
            } else {
                let l_win = archvnde_launcher::widgets::build_launcher_ui(&app_clone, lw_clone.clone());
                l_win.present();
                if let Ok(mut borrow) = lw_clone.try_borrow_mut() {
                    *borrow = Some(l_win);
                }
            }
        });

        // 2. Workspace Switcher
        let workspace_box = create_workspace_switcher();

        // Create a separator to visual separate logo and dots inside the same capsule
        let separator = gtk4::Label::new(Some("│"));
        separator.add_css_class("capsule-separator");

        workspace_box.prepend(&separator);
        workspace_box.prepend(&logo_btn);

        // 3. Unified Status and Clock Capsule
        let status_indicators = create_status_indicators(
            app,
            control_center_window.clone(),
            calendar_window.clone(),
            launcher_window.clone(),
        );

        let sys_monitor = create_sys_monitor_widget();
        let tray_widget = create_tray_widget();

        // Left-aligned section: Workspaces capsule (now containing logo + separator + dots)
        let left_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        left_box.set_hexpand(true);
        left_box.set_halign(gtk4::Align::Start);
        left_box.set_valign(gtk4::Align::Center);
        left_box.append(&workspace_box);

        let left_wrapper = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        left_wrapper.set_valign(gtk4::Align::Start);
        left_wrapper.set_size_request(-1, 35);
        left_wrapper.append(&left_box);

        // Center-aligned section: Clean placeholder center space with interactive notch
        let center_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        center_box.set_hexpand(true);
        center_box.set_halign(gtk4::Align::Center);
        center_box.set_valign(gtk4::Align::Start);

        let notch_capsule = create_system_island();
        center_box.append(&notch_capsule);

        // Right-aligned section: Status & Clock capsule
        let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        right_box.set_hexpand(true);
        right_box.set_halign(gtk4::Align::End);
        right_box.set_valign(gtk4::Align::Center);
        right_box.append(&tray_widget);
        right_box.append(&sys_monitor);
        right_box.append(&status_indicators);

        let right_wrapper = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        right_wrapper.set_valign(gtk4::Align::Start);
        right_wrapper.set_size_request(-1, 35);
        right_wrapper.append(&right_box);

        // Assemble columns into the main panel box using CenterBox
        box_layout.set_start_widget(Some(&left_wrapper));
        box_layout.set_center_widget(Some(&center_box));
        box_layout.set_end_widget(Some(&right_wrapper));

        window.set_child(Some(&box_layout));

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
