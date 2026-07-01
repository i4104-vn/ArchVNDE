use archvnde_common::desktop::DesktopApp;
use crate::history::get_history;

pub fn get_running_apps() -> Vec<DesktopApp> {
    let desktop_apps = archvnde_common::desktop::find_desktop_apps();
    let mut running = Vec::new();
    let mut detected_windows = std::collections::HashSet::new();

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
    //    Each window is a separate entry (no dedup by app name)
    for (app_id, title) in running_windows {
        let app_id_lower = app_id.to_lowercase();
        let title_lower = title.to_lowercase();
        let mut matched_app = None;

        // Use app_id + title as unique window key to avoid exact duplicates
        let window_key = format!("{}::{}", app_id, title);
        if detected_windows.contains(&window_key) {
            continue;
        }
        detected_windows.insert(window_key);

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
            app.app_id = Some(app_id.clone());
            app.window_title = Some(title.clone());
            running.push(app);
        } else {
            // If still no desktop file found, construct a placeholder app
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

    // 3. Sort running apps based on MRU history (use window_title for per-window tracking)
    let history = get_history();
    running.sort_by(|a, b| {
        let key_a = a.window_title.as_deref().unwrap_or(&a.name);
        let key_b = b.window_title.as_deref().unwrap_or(&b.name);
        let idx_a = history.iter().position(|x| x == key_a).unwrap_or(usize::MAX);
        let idx_b = history.iter().position(|x| x == key_b).unwrap_or(usize::MAX);
        idx_a.cmp(&idx_b)
    });

    running
}

pub fn activate_app(app: &DesktopApp) {
    // 1. Try to focus using the exact window title first (most specific, works for multi-window apps)
    if let Some(ref title) = app.window_title {
        println!("Activating by window title: {}", title);
        let status = std::process::Command::new("wlrctl")
            .args(&["window", "focus", &format!("title:{}", title)])
            .status();
        if let Ok(s) = status {
            if s.success() {
                return;
            }
        }
        
        // Try shorter title matches by splitting on common delimiters
        for delim in &[" — ", " - "] {
            if let Some(pos) = title.rfind(delim) {
                let short_title = title[..pos].trim();
                if !short_title.is_empty() {
                    let status = std::process::Command::new("wlrctl")
                        .args(&["window", "focus", &format!("title:{}", short_title)])
                        .status();
                    if let Ok(s) = status {
                        if s.success() {
                            return;
                        }
                    }
                }
            }
        }
    }

    // 2. Try to focus using the app_id from Wayland
    if let Some(ref app_id) = app.app_id {
        println!("Activating by app_id: {}", app_id);
        let status = std::process::Command::new("wlrctl")
            .args(&["window", "focus", app_id])
            .status();
        if let Ok(s) = status {
            if s.success() {
                return;
            }
        }
        let status_lower = std::process::Command::new("wlrctl")
            .args(&["window", "focus", &app_id.to_lowercase()])
            .status();
        if let Ok(s) = status_lower {
            if s.success() {
                return;
            }
        }
    }

    // 3. Fallbacks: Try generic matching by name and executable name
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

    if !exec_name.is_empty() {
        let _ = std::process::Command::new("wlrctl")
            .args(&["window", "focus", &exec_name])
            .status();
        let _ = std::process::Command::new("wlrctl")
            .args(&["window", "focus", &format!("title:{}", exec_name)])
            .status();
    }
    
    if !app.exec.is_empty() {
        let _ = std::process::Command::new("wlrctl")
            .args(&["window", "focus", &app.exec])
            .status();
    }

    let _ = std::process::Command::new("wlrctl")
        .args(&["window", "focus", name])
        .status();
    let _ = std::process::Command::new("wlrctl")
        .args(&["window", "focus", &format!("title:{}", name)])
        .status();
}

