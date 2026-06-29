use archvnde_common::desktop::DesktopApp;
use crate::history::get_history;

pub fn get_running_apps() -> Vec<DesktopApp> {
    let desktop_apps = archvnde_common::desktop::find_desktop_apps();
    let mut running = Vec::new();
    let mut detected_names = std::collections::HashSet::new();

    // 1. Get all running process names from /proc
    let mut running_processes = std::collections::HashSet::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.chars().all(|c| c.is_ascii_digit()) {
                        let comm_path = entry.path().join("comm");
                        if let Ok(comm) = std::fs::read_to_string(comm_path) {
                            let process_name = comm.trim().to_lowercase();
                            if !process_name.is_empty() {
                                running_processes.insert(process_name);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Match running processes with desktop entries
    for app in desktop_apps {
        let exec_parts: Vec<&str> = app.exec.split_whitespace().collect();
        if exec_parts.is_empty() {
            continue;
        }
        let exec_path = std::path::Path::new(exec_parts[0]);
        let exec_name = exec_path.file_name()
            .map(|f| f.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if exec_name.is_empty() {
            continue;
        }

        if running_processes.contains(&exec_name) {
            let app_key = exec_name.clone();
            if !detected_names.contains(&app_key) {
                detected_names.insert(app_key);
                running.push(app);
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

    // Try wlrctl (Wayland wlroots)
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
}
