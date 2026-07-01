mod apps;
mod preview;

use gtk4::prelude::*;
use archvnde_common::desktop::DesktopApp;
use apps::get_running_windows;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct PopoverState {
    popover: gtk4::Popover,
}

fn get_active_app_id() -> Option<String> {
    if let Ok(output) = std::process::Command::new("wlrctl")
        .args(&["window", "list", "state:focused"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            if let Some(pos) = line.find(':') {
                let app_id = line[..pos].trim().to_string();
                if !app_id.is_empty() {
                    return Some(app_id);
                }
            }
        }
    }
    None
}

/// Helper to generate a signature representing current taskbar state.
/// This signature changes when applications are opened/closed, or when the active app changes.
fn get_taskbar_signature(running_apps: &[DesktopApp], active_app_id: Option<&str>) -> String {
    let mut counts = HashMap::new();
    for app in running_apps {
        let app_id = app.app_id.clone().unwrap_or_else(|| app.name.clone());
        *counts.entry(app_id).or_insert(0) += 1;
    }
    let mut sigs = Vec::new();
    for (app_id, count) in counts {
        sigs.push(format!("{}:{}", app_id, count));
    }
    sigs.sort();
    let apps_sig = sigs.join("||");
    format!("{}//active:{}", apps_sig, active_app_id.unwrap_or(""))
}

/// Dynamic rebuild of the taskbar buttons
fn rebuild_taskbar(
    apps_box: &gtk4::Box,
    running_apps: Vec<DesktopApp>,
    active_app_id: Option<String>,
    popovers: &Rc<RefCell<Vec<PopoverState>>>,
    running_apps_shared: Arc<Mutex<(Vec<DesktopApp>, Option<String>)>>,
) {
    // 1. Unparent all previous popovers first to prevent crashes
    for state in popovers.borrow_mut().drain(..) {
        state.popover.unparent();
    }

    // 2. Clear all existing children from the apps box container
    while let Some(child) = apps_box.first_child() {
        apps_box.remove(&child);
    }

    if running_apps.is_empty() {
        return;
    }

    // 3. Group running apps by app_id
    let mut groups: HashMap<String, Vec<DesktopApp>> = HashMap::new();
    for app in running_apps {
        let app_id = app.app_id.clone().unwrap_or_else(|| app.name.clone());
        groups.entry(app_id).or_default().push(app);
    }

    // 4. Sort groups alphabetically by app_id to prevent buttons from shifting places randomly
    let mut group_keys: Vec<String> = groups.keys().cloned().collect();
    group_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    // 5. Build buttons for each application group
    for app_id in group_keys {
        let windows = groups.get(&app_id).unwrap();
        let first_app = &windows[0];

        let btn = gtk4::Button::new();
        btn.add_css_class("taskbar-app-btn");
        btn.set_valign(gtk4::Align::Center);

        // Highlight active app
        if let Some(ref active_id) = active_app_id {
            let active_id_lower = active_id.to_lowercase();
            let is_active = app_id.to_lowercase() == active_id_lower || windows.iter().any(|w| {
                w.app_id.as_ref().map(|id| id.to_lowercase() == active_id_lower).unwrap_or(false)
            });
            if is_active {
                btn.add_css_class("active");
            }
        }

        // Load app icon
        let icon_name = first_app.icon.clone().unwrap_or_else(|| app_id.clone());
        let icon = archvnde_common::icon::get_icon(&icon_name, 14);
        btn.set_child(Some(&icon));

        // Create Popover for window previews
        let popover = gtk4::Popover::new();
        popover.add_css_class("taskbar-popover");
        popover.set_parent(&btn);
        popover.set_position(gtk4::PositionType::Bottom);
        popover.set_has_arrow(true);

        // Setup click action with on-demand list generation
        let pop_clone = popover.clone();
        let app_id_clone = app_id.clone();
        let apps_shared = running_apps_shared.clone();
        btn.connect_clicked(move |_| {
            // Dynamically build list on click from the background thread's latest window list
            let windows = if let Ok(lock) = apps_shared.lock() {
                lock.0.clone()
            } else {
                Vec::new()
            };
            let app_windows: Vec<DesktopApp> = windows.into_iter()
                .filter(|w| {
                    let w_id = w.app_id.as_deref().unwrap_or(&w.name);
                    w_id == app_id_clone
                })
                .collect();
            
            if app_windows.len() > 1 {
                preview::populate_popover_previews(&pop_clone, &app_windows, &app_id_clone);
                pop_clone.popup();
            } else if let Some(single_window) = app_windows.first() {
                let target_app_id = single_window.app_id.as_deref().unwrap_or(&app_id_clone);
                let target_title = single_window.window_title.as_deref().unwrap_or("");
                apps::focus_window(target_app_id, target_title);
            }
        });

        popovers.borrow_mut().push(PopoverState {
            popover,
        });

        apps_box.append(&btn);
    }
}

/// Creates and returns a taskbar container showing running windows grouped by app class.
pub fn create_workspace_switcher() -> gtk4::Box {
    let parent_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    parent_box.add_css_class("taskbar-parent-box");

    let apps_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    apps_box.add_css_class("taskbar-apps-box");

    parent_box.append(&apps_box);

    let popovers = Rc::new(RefCell::new(Vec::new()));
    let last_signature = Rc::new(RefCell::new(String::new()));

    // Shared thread-safe state for running windows and active app_id
    let running_apps_shared = Arc::new(Mutex::new((Vec::new(), None)));

    // Spawn background thread to query wlrctl asynchronously (completely eliminates lag/micro-stutters!)
    let apps_shared_clone = running_apps_shared.clone();
    thread::spawn(move || {
        loop {
            let apps = get_running_windows();
            let active_app_id = get_active_app_id();
            if let Ok(mut lock) = apps_shared_clone.lock() {
                *lock = (apps, active_app_id);
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    // GTK main thread loop reads from the shared state (non-blocking!)
    let apps_box_clone = apps_box.clone();
    let popovers_clone = popovers.clone();
    let sig_clone = last_signature.clone();
    let apps_shared_for_timer = running_apps_shared.clone();
    let apps_shared_for_rebuild = running_apps_shared.clone();
    
    // Initial delay load
    glib::timeout_add_local_once(Duration::from_millis(200), {
        let apps_box = apps_box_clone.clone();
        let popovers = popovers_clone.clone();
        let sig = sig_clone.clone();
        let apps_shared = running_apps_shared.clone();
        let apps_shared_rebuild = apps_shared_for_rebuild.clone();
        move || {
            let (apps, active_app_id) = if let Ok(lock) = apps_shared.lock() {
                lock.clone()
            } else {
                (Vec::new(), None)
            };
            *sig.borrow_mut() = get_taskbar_signature(&apps, active_app_id.as_deref());
            rebuild_taskbar(&apps_box, apps, active_app_id, &popovers, apps_shared_rebuild);
        }
    });

    glib::timeout_add_local(Duration::from_millis(500), move || {
        let (apps, active_app_id) = if let Ok(lock) = apps_shared_for_timer.lock() {
            lock.clone()
        } else {
            (Vec::new(), None)
        };
        let new_sig = get_taskbar_signature(&apps, active_app_id.as_deref());
        
        if new_sig != *sig_clone.borrow() {
            *sig_clone.borrow_mut() = new_sig;
            rebuild_taskbar(&apps_box_clone, apps, active_app_id, &popovers_clone, apps_shared_for_rebuild.clone());
        }
        glib::ControlFlow::Continue
    });

    parent_box
}
