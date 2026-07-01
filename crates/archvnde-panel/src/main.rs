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

fn rebuild_panel_window(
    window: &gtk4::ApplicationWindow,
    app: &gtk4::Application,
    control_center_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    launcher_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) {
    // 1. Remove child
    window.set_child(None::<&gtk4::Widget>);

    // 2. Layout container
    let box_layout = gtk4::CenterBox::new();
    box_layout.add_css_class("panel-box");

    // 3. Logo Button
    let logo_btn = gtk4::Button::new();
    logo_btn.add_css_class("panel-logo-btn");
    let logo_icon = archvnde_common::icon::get_icon("logo", 16);
    logo_btn.set_child(Some(&logo_icon));
    
    let lw_clone = launcher_window.clone();
    let ccw_clone = control_center_window.clone();
    let cw_clone = calendar_window.clone();
    let app_clone = app.clone();
    logo_btn.connect_clicked(move |_| {
        let cc_win = { ccw_clone.borrow().clone() };
        if let Some(win) = cc_win { win.close(); }
        let cal_win = { cw_clone.borrow().clone() };
        if let Some(win) = cal_win { win.close(); }
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

    // 4. Workspace Switcher
    let workspace_box = create_workspace_switcher();
    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");
    workspace_box.prepend(&separator);
    workspace_box.prepend(&logo_btn);

    // 5. Unified Status and Clock Capsule
    let status_indicators = create_status_indicators(
        app,
        control_center_window.clone(),
        calendar_window.clone(),
        launcher_window.clone(),
    );

    let sys_monitor = create_sys_monitor_widget();
    let tray_widget = create_tray_widget(window);

    // Left Wrapper
    let left_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    left_box.set_hexpand(true);
    left_box.set_halign(gtk4::Align::Start);
    left_box.set_valign(gtk4::Align::Center);
    left_box.append(&workspace_box);

    let left_wrapper = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    left_wrapper.set_valign(gtk4::Align::Start);
    left_wrapper.set_size_request(-1, 35);
    left_wrapper.append(&left_box);

    // Center Wrapper
    let center_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    center_box.set_hexpand(true);
    center_box.set_halign(gtk4::Align::Center);
    center_box.set_valign(gtk4::Align::Start);

    let notch_capsule = create_system_island();
    center_box.append(&notch_capsule);

    // Right Wrapper
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

    // Assemble CenterBox
    box_layout.set_start_widget(Some(&left_wrapper));
    box_layout.set_center_widget(Some(&center_box));
    box_layout.set_end_widget(Some(&right_wrapper));

    window.set_child(Some(&box_layout));
}

fn main() {
    println!("Starting ArchVNDE Panel...");

    // Initialize D-Bus StatusNotifierWatcher system tray listener daemon
    archvnde_tray::spawn_watcher_service();

    // Spawn a background thread to refresh desktop apps cache asynchronously on startup
    std::thread::spawn(|| {
        println!("Background thread refreshing desktop apps cache...");
        archvnde_common::refresh_desktop_apps_cache();
        println!("Background desktop apps cache refresh complete.");
    });

    // Spawn background thread to track focused app and capture screenshots
    std::thread::spawn(|| {
        use std::process::Command;
        use std::time::{Instant, Duration};
        use std::fs;

        let cache_dir = "/tmp/archvnde-switcher-cache";
        let _ = fs::create_dir_all(cache_dir);

        let mut current_focused_window: Option<(String, String)> = None;
        let mut focus_start = Instant::now();
        let mut screenshot_taken = false;

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let switcher_open = std::path::Path::new("/tmp/archvnde-switcher.socket").exists();
            if switcher_open {
                if let Some((ref old_app, ref old_title)) = current_focused_window {
                    if screenshot_taken {
                        let temp_file = format!("{}/temp_active.png", cache_dir);
                        // Save window-specific screenshot
                        let hash = archvnde_common::desktop::get_window_hash(old_app, old_title);
                        let dest_file = format!("{}/{}.png", cache_dir, hash);
                        let _ = fs::copy(&temp_file, &dest_file);
                        // Save generic fallback screenshot
                        let dest_generic = format!("{}/{}.png", cache_dir, old_app);
                        let _ = fs::copy(&temp_file, &dest_generic);
                    }
                }
                current_focused_window = None;
                screenshot_taken = false;
                continue;
            }

            // Run wlrctl to get the currently focused window
            let output = Command::new("wlrctl")
                .args(&["window", "list", "state:focused"])
                .output();

            let active_window = if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = stdout.lines().next() {
                    if let Some(pos) = line.find(':') {
                        let app_id = line[..pos].trim().to_string();
                        let title = line[pos + 1..].trim().to_string();
                        if !app_id.is_empty() {
                            Some((app_id, title))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Ignore the switcher itself if it gets focused
            let is_switcher = active_window.as_ref().map(|(s, _)| s == "archvnde-switcher" || s == "org.archvnde.switcher").unwrap_or(false);
            if is_switcher {
                continue;
            }

            if active_window != current_focused_window {
                // User switched away from current_focused_window
                if let Some((ref old_app, ref old_title)) = current_focused_window {
                    if screenshot_taken {
                        // Copy the temp screenshot to the old window's cache file
                        let temp_file = format!("{}/temp_active.png", cache_dir);
                        let hash = archvnde_common::desktop::get_window_hash(old_app, old_title);
                        let dest_file = format!("{}/{}.png", cache_dir, hash);
                        let _ = fs::copy(&temp_file, &dest_file);
                        // Copy to generic fallback screenshot
                        let dest_generic = format!("{}/{}.png", cache_dir, old_app);
                        let _ = fs::copy(&temp_file, &dest_generic);
                    }
                }

                // Clean up stale cache files for windows that are no longer running
                if let Ok(entries) = fs::read_dir(cache_dir) {
                    let mut running_hashes = std::collections::HashSet::new();
                    let mut running_app_ids = std::collections::HashSet::new();
                    if let Ok(out) = Command::new("wlrctl").args(&["window", "list"]).output() {
                        let list_stdout = String::from_utf8_lossy(&out.stdout);
                        for line in list_stdout.lines() {
                            if let Some(pos) = line.find(':') {
                                let id = line[..pos].trim().to_string();
                                let title = line[pos + 1..].trim().to_string();
                                if !id.is_empty() {
                                    running_hashes.insert(archvnde_common::desktop::get_window_hash(&id, &title));
                                    running_app_ids.insert(id);
                                }
                            }
                        }
                    }
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(file_name) = path.file_name() {
                                    let name_str = file_name.to_string_lossy().to_string();
                                    if name_str != "temp_active.png" && name_str.ends_with(".png") {
                                        let key = name_str.trim_end_matches(".png").to_string();
                                        if !running_hashes.contains(&key) && !running_app_ids.contains(&key) {
                                            let _ = fs::remove_file(&path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Reset for the new active window
                current_focused_window = active_window;
                focus_start = Instant::now();
                screenshot_taken = false;
            } else if current_focused_window.is_some() && !screenshot_taken {
                // If they have stayed in the same window for >= 5 seconds, take a screenshot
                if focus_start.elapsed() >= Duration::from_secs(5) {
                    let temp_file = format!("{}/temp_active.png", cache_dir);
                    // Run grim to capture the screen
                    let status = Command::new("grim")
                        .arg(&temp_file)
                        .status();
                    if let Ok(s) = status {
                        if s.success() {
                            screenshot_taken = true;
                        }
                    }
                }
            }
        }
    });

    let application = gtk4::Application::new(
        Some("org.archvnde.panel"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);
        archvnde_common::apply_theme_class(&window);

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

        let window_c = window.clone();
        let app_c = app.clone();
        let ccw_c = control_center_window.clone();
        let cw_c = calendar_window.clone();
        let lw_c = launcher_window.clone();

        rebuild_panel_window(
            &window, 
            app, 
            control_center_window.clone(), 
            calendar_window.clone(), 
            launcher_window.clone()
        );

        if let Some(settings) = gtk4::Settings::default() {
            settings.connect_gtk_application_prefer_dark_theme_notify(move |_| {
                rebuild_panel_window(
                    &window_c, 
                    &app_c, 
                    ccw_c.clone(), 
                    cw_c.clone(), 
                    lw_c.clone()
                );
            });
        }

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
