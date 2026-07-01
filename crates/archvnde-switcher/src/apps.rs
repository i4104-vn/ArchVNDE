use archvnde_common::desktop::DesktopApp;
use crate::history::get_history;

pub fn get_running_apps() -> Vec<DesktopApp> {
    let desktop_apps = archvnde_common::desktop::find_desktop_apps();
    let mut running = Vec::new();
    let mut detected_names = std::collections::HashSet::new();

    // 1. Get running app_ids and titles from `wlrctl toplevel list` (retain original casing!)
    let mut running_windows = Vec::new();
    if let Ok(output) = std::process::Command::new("wlrctl").args(&["toplevel", "list"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(pos) = line.find(':') {
                let app_id = line[..pos].trim().to_string();
                let title = line[pos + 1..].trim().to_string();
                if !app_id.is_empty() {
                    running_windows.push((app_id, title));
                }
            }
        }
    }

    // 2. Match running windows with desktop entries
    for (app_id, title) in running_windows {
        let app_id_lower = app_id.to_lowercase();
        let title_lower = title.to_lowercase();
        let mut matched_app = None;

        // Pass 1: Try to match app_id with executable name or desktop file name or app name
        for app in &desktop_apps {
            let exec_parts: Vec<&str> = app.exec.split_whitespace().collect();
            if exec_parts.is_empty() {
                continue;
            }
            let exec_path = std::path::Path::new(exec_parts[0]);
            let exec_name = exec_path.file_name()
                .map(|f| f.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if exec_name == app_id_lower || app.name.to_lowercase() == app_id_lower {
                matched_app = Some(app.clone());
                break;
            }
        }

        // Pass 2: If no match, try substring matching
        if matched_app.is_none() {
            for app in &desktop_apps {
                let exec_parts: Vec<&str> = app.exec.split_whitespace().collect();
                if exec_parts.is_empty() {
                    continue;
                }
                let exec_path = std::path::Path::new(exec_parts[0]);
                let exec_name = exec_path.file_name()
                    .map(|f| f.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                if app_id_lower.contains(&exec_name) || exec_name.contains(&app_id_lower) || 
                   title_lower.contains(&app.name.to_lowercase()) || app.name.to_lowercase().contains(&app_id_lower) {
                    matched_app = Some(app.clone());
                    break;
                }
            }
        }

        // Special fallback for well-known apps (like Navigator -> Firefox)
        if matched_app.is_none() && (app_id_lower == "navigator" || title_lower.contains("firefox")) {
            for app in &desktop_apps {
                if app.name.to_lowercase().contains("firefox") {
                    matched_app = Some(app.clone());
                    break;
                }
            }
        }

        if let Some(mut app) = matched_app {
            app.app_id = Some(app_id.clone()); // Store original case
            app.window_title = Some(title.clone()); // Store original case
            let app_key = app.name.clone();
            if !detected_names.contains(&app_key) {
                detected_names.insert(app_key);
                running.push(app);
            }
        } else {
            // If still no desktop file found, construct a placeholder app so it still shows in Alt-Tab!
            let app_key = app_id.clone();
            if !detected_names.contains(&app_key) {
                detected_names.insert(app_key.clone());
                
                // Capitalize app_id for displaying name
                let mut chars = app_id.chars();
                let display_name = match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                };
                
                running.push(DesktopApp {
                    name: display_name,
                    exec: app_id.clone(),
                    icon: Some(app_id.clone()),
                    app_id: Some(app_id.clone()),
                    window_title: Some(title.clone()),
                });
            }
        }
    }

    // 3. Sort running apps based on MRU history
    let history = get_history();
    running.sort_by(|a, b| {
        let idx_a = history.iter().position(|x| x == &a.name).unwrap_or(usize::MAX);
        let idx_b = history.iter().position(|x| x == &b.name).unwrap_or(usize::MAX);
        idx_a.cmp(&idx_b)
    });

    running
}

pub fn activate_app(app: &DesktopApp) {
    // 1. Try to focus using the exact app_id from Wayland if we captured it
    if let Some(ref app_id) = app.app_id {
        println!("Activating by app_id: {}", app_id);
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", app_id])
            .spawn();
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &app_id.to_lowercase()])
            .spawn();
    }

    // 2. Try to focus using the exact window title from Wayland if we captured it
    if let Some(ref title) = app.window_title {
        println!("Activating by window title: {}", title);
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &format!("title:{}", title)])
            .spawn();
        // Also split by common delimiters to try shorter matches
        if let Some(pos) = title.rfind(" — ") {
            let short_title = &title[..pos].trim();
            if !short_title.is_empty() {
                let _ = std::process::Command::new("wlrctl")
                    .args(&["toplevel", "focus", &format!("title:{}", short_title)])
                    .spawn();
            }
        }
        if let Some(pos) = title.rfind(" - ") {
            let short_title = &title[..pos].trim();
            if !short_title.is_empty() {
                let _ = std::process::Command::new("wlrctl")
                    .args(&["toplevel", "focus", &format!("title:{}", short_title)])
                    .spawn();
            }
        }
    }

    // Fallbacks: Try generic matching by name and executable name
    let name = &app.name;
    let exec_parts: Vec<&str> = app.exec.split_whitespace().collect();
    let exec_name = if !exec_parts.is_empty() {
        std::path::Path::new(exec_parts[0])
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Try wlrctl (Wayland wlroots) using app_id/exec_name
    if !exec_name.is_empty() {
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &exec_name])
            .spawn();
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &exec_name.to_lowercase()])
            .spawn();
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &format!("title:{}", exec_name)])
            .spawn();
    }
    
    // Also try the raw app.exec (covers placeholder app_ids)
    if !app.exec.is_empty() {
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &app.exec])
            .spawn();
        let _ = std::process::Command::new("wlrctl")
            .args(&["toplevel", "focus", &app.exec.to_lowercase()])
            .spawn();
    }

    let _ = std::process::Command::new("wlrctl")
        .args(&["toplevel", "focus", name])
        .spawn();
    let _ = std::process::Command::new("wlrctl")
        .args(&["toplevel", "focus", &format!("title:{}", name)])
        .spawn();
    let _ = std::process::Command::new("wlrctl")
        .args(&["toplevel", "focus", &format!("title:{}", name.to_lowercase())])
        .spawn();
    // Try wmctrl (X11 / XWayland)
    let _ = std::process::Command::new("wmctrl")
        .args(&["-a", name])
        .spawn();
    if !exec_name.is_empty() {
        let _ = std::process::Command::new("wmctrl")
            .args(&["-a", &exec_name])
            .spawn();
    }

    // Spawn a thread to send a dummy Ctrl press and release to cancel the Alt menu bar trigger
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(150));
        let home = std::env::var("HOME").unwrap_or_default();
        let wtype_path = if !home.is_empty() {
            let path = format!("{}/.local/bin/wtype", home);
            if std::path::Path::new(&path).exists() {
                path
            } else {
                "wtype".to_string()
            }
        } else {
            "wtype".to_string()
        };

        let _ = std::process::Command::new(wtype_path)
            .args(&["-M", "ctrl", "-m", "ctrl"])
            .status();
    });
}
