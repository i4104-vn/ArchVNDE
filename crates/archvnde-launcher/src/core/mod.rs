use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::models::DesktopApp;

/// Scans standard Linux directories for `.desktop` files, parses their entries,
/// and returns a sorted, deduplicated list of launchable desktop applications.
pub fn find_desktop_apps() -> Vec<DesktopApp> {
    let mut apps = Vec::new();
    let paths = vec![
        PathBuf::from("/usr/share/applications"),
        dirs::data_dir()
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".local/share")
            })
            .join("applications"),
    ];

    for path in paths {
        if !path.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().map(|e| e == "desktop").unwrap_or(false) {
                    if let Some(app) = parse_desktop_file(&entry_path) {
                        apps.push(app);
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());

    apps
}

/// Parses a single Linux `.desktop` configuration file to extract the application name,
/// execution command (excluding field code arguments like `%u` or `%F`), and icon name.
/// Returns None if the application has a `NoDisplay=true` entry.
fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut no_display = false;
    let mut in_desktop_entry = false;

    for line in reader.lines().flatten() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
            } else {
                in_desktop_entry = false;
            }
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();

            match key {
                "Name" if name.is_none() => name = Some(value.to_string()),
                "Exec" if exec.is_none() => {
                    let clean_exec = value
                        .split_whitespace()
                        .filter(|word| !word.starts_with('%'))
                        .collect::<Vec<&str>>()
                        .join(" ");
                    exec = Some(clean_exec);
                }
                "Icon" if icon.is_none() => icon = Some(value.to_string()),
                "NoDisplay" => {
                    if value.to_lowercase() == "true" {
                        no_display = true;
                    }
                }
                _ => {}
            }
        }
    }

    if no_display {
        return None;
    }

    match (name, exec) {
        (Some(n), Some(e)) => Some(DesktopApp { name: n, exec: e, icon }),
        _ => None,
    }
}
