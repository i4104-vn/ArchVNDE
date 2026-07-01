use gdk4::Texture;
use gdk_pixbuf::Pixbuf;

pub const WIFI_SVG: &str = include_str!("assets/wifi.svg");
pub const BLUETOOTH_SVG: &str = include_str!("assets/bluetooth.svg");
pub const PERFORMANCE_SVG: &str = include_str!("assets/performance.svg");
pub const NIGHT_LIGHT_SVG: &str = include_str!("assets/night-light.svg");
pub const DARK_MODE_SVG: &str = include_str!("assets/dark-mode.svg");
pub const CAFFEINE_SVG: &str = include_str!("assets/caffeine.svg");
pub const GSCONNECT_SVG: &str = include_str!("assets/gsconnect.svg");
pub const PRIVACY_SVG: &str = include_str!("assets/privacy.svg");
pub const SETTINGS_SVG: &str = include_str!("assets/settings.svg");
pub const POWER_SVG: &str = include_str!("assets/power.svg");
pub const VOLUME_SVG: &str = include_str!("assets/volume.svg");
pub const BRIGHTNESS_SVG: &str = include_str!("assets/brightness.svg");
pub const BATTERY_SVG: &str = include_str!("assets/battery.svg");
pub const LOGO_SVG: &str = include_str!("assets/logo.svg");
pub const MICROPHONE_SVG: &str = include_str!("assets/microphone.svg");
pub const AIRPLANE_SVG: &str = include_str!("assets/airplane.svg");
pub const LOCK_SVG: &str = include_str!("assets/lock.svg");
pub const LOGOUT_SVG: &str = include_str!("assets/logout.svg");
pub const RESTART_SVG: &str = include_str!("assets/restart.svg");
pub const BELL_SVG: &str = include_str!("assets/bell.svg");
pub const DISPLAY_SVG: &str = include_str!("assets/display.svg");
pub const USER_SVG: &str = include_str!("assets/user.svg");
pub const FOLDER_SVG: &str = include_str!("assets/folder.svg");
pub const TERMINAL_SVG: &str = include_str!("assets/terminal.svg");
pub const CAMERA_SVG: &str = include_str!("assets/camera.svg");
pub const CLOCK_SVG: &str = include_str!("assets/clock.svg");
pub const SEARCH_SVG: &str = include_str!("assets/search.svg");
pub const MUSIC_SVG: &str = include_str!("assets/music.svg");
pub const ETHERNET_SVG: &str = include_str!("assets/ethernet.svg");
pub const UNLOCK_SVG: &str = include_str!("assets/unlock.svg");
pub const TRASH_SVG: &str = include_str!("assets/trash.svg");
pub const BELL_OFF_SVG: &str = include_str!("assets/bell-off.svg");
pub const ACTIVITY_SVG: &str = include_str!("assets/activity.svg");
pub const TEXT_SVG: &str = include_str!("assets/text.svg");
pub const SERVER_SVG: &str = include_str!("assets/server.svg");
pub const DOWNLOAD_SVG: &str = include_str!("assets/download.svg");
pub const SHIELD_SVG: &str = include_str!("assets/shield.svg");
pub const INFO_SVG: &str = include_str!("assets/info.svg");
pub const PLUS_SVG: &str = include_str!("assets/plus.svg");

/// Loads an SVG string into a GTK4 Image widget at a custom size.
pub fn get_icon_from_svg(svg_content: &str, size: i32) -> gtk4::Image {
    let bytes = glib::Bytes::from(svg_content.as_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    
    let pixbuf = Pixbuf::from_stream_at_scale(
        &stream,
        size,
        size,
        true,
        gio::Cancellable::NONE
    );

    match pixbuf {
        Ok(pb) => {
            let texture = Texture::for_pixbuf(&pb);
            let img = gtk4::Image::from_paintable(Some(&texture));
            img.set_pixel_size(size);
            img
        }
        Err(_) => {
            gtk4::Image::from_icon_name("image-missing")
        }
    }
}

/// Robust helper to determine whether dark mode is currently active system-wide.
pub fn is_dark_mode() -> bool {
    if let Some(settings) = gtk4::Settings::default() {
        settings.is_gtk_application_prefer_dark_theme()
    } else {
        true
    }
}

/// Helper function to retrieve an SVG icon widget by name with a custom stroke color.
pub fn get_icon_colored(name: &str, size: i32, color_hex: &str) -> gtk4::Image {
    let is_dark = is_dark_mode();

    let resolved_color = if !is_dark {
        let clean = color_hex.trim().to_lowercase();
        if clean.starts_with("rgba(255, 255, 255,") || clean.starts_with("rgba(255,255,255,") {
            let alpha = clean.split(',').last().unwrap_or("1.0)").trim().replace(")", "");
            format!("rgba(28, 28, 30, {})", alpha)
        } else {
            color_hex.to_string()
        }
    } else {
        color_hex.to_string()
    };

    let svg = match name {
        "wifi" => Some(WIFI_SVG),
        "bluetooth" => Some(BLUETOOTH_SVG),
        "performance" => Some(PERFORMANCE_SVG),
        "night-light" => Some(NIGHT_LIGHT_SVG),
        "dark-mode" => Some(DARK_MODE_SVG),
        "caffeine" => Some(CAFFEINE_SVG),
        "gsconnect" => Some(GSCONNECT_SVG),
        "privacy" => Some(PRIVACY_SVG),
        "settings" => Some(SETTINGS_SVG),
        "power" => Some(POWER_SVG),
        "volume" => Some(VOLUME_SVG),
        "brightness" => Some(BRIGHTNESS_SVG),
        "battery" => Some(BATTERY_SVG),
        "logo" => Some(LOGO_SVG),
        "microphone" => Some(MICROPHONE_SVG),
        "airplane" => Some(AIRPLANE_SVG),
        "lock" => Some(LOCK_SVG),
        "logout" => Some(LOGOUT_SVG),
        "restart" => Some(RESTART_SVG),
        "bell" => Some(BELL_SVG),
        "display" => Some(DISPLAY_SVG),
        "user" => Some(USER_SVG),
        "folder" => Some(FOLDER_SVG),
        "terminal" => Some(TERMINAL_SVG),
        "camera" => Some(CAMERA_SVG),
        "clock" => Some(CLOCK_SVG),
        "search" => Some(SEARCH_SVG),
        "music" => Some(MUSIC_SVG),
        "ethernet" => Some(ETHERNET_SVG),
        "unlock" => Some(UNLOCK_SVG),
        "trash" => Some(TRASH_SVG),
        "bell-off" => Some(BELL_OFF_SVG),
        "activity" => Some(ACTIVITY_SVG),
        "text" => Some(TEXT_SVG),
        "server" => Some(SERVER_SVG),
        "download" => Some(DOWNLOAD_SVG),
        "shield" => Some(SHIELD_SVG),
        "info" => Some(INFO_SVG),
        "plus" => Some(PLUS_SVG),
        _ => None,
    };

    if let Some(svg_content) = svg {
        let colored_svg = svg_content.replace("currentColor", &resolved_color);
        get_icon_from_svg(&colored_svg, size)
    } else {
        let img = get_system_or_file_icon(name, "image-missing");
        img.set_pixel_size(size);
        img
    }
}

/// Helper function to retrieve an SVG icon widget by name. Defaults to white in dark mode and dark gray in light mode.
pub fn get_icon(name: &str, size: i32) -> gtk4::Image {
    let is_dark = is_dark_mode();
    let color = if is_dark { "#ffffff" } else { "#1c1c1e" };
    get_icon_colored(name, size, color)
}

/// Loads a system icon by name or from a local absolute file path, with robust theme validation and desktop resolution.
pub fn get_system_or_file_icon(icon_path_or_name: &str, default_fallback: &str) -> gtk4::Image {
    if icon_path_or_name.is_empty() {
        return gtk4::Image::from_icon_name(default_fallback);
    }
    
    if icon_path_or_name.starts_with('/') {
        return gtk4::Image::from_file(icon_path_or_name);
    }
    
    let mut clean_name = icon_path_or_name.to_string();
    for ext in &[".png", ".svg", ".xpm", ".jpg", ".jpeg", ".gif"] {
        if clean_name.to_lowercase().ends_with(ext) {
            clean_name = clean_name[..clean_name.len() - ext.len()].to_string();
            break;
        }
    }

    let display = gdk4::Display::default();
    let has_icon = if let Some(ref disp) = display {
        let theme = gtk4::IconTheme::for_display(disp);
        theme.has_icon(&clean_name)
    } else {
        false
    };

    if has_icon {
        gtk4::Image::from_icon_name(&clean_name)
    } else {
        let lower_name = clean_name.to_lowercase();
        let apps = crate::core::desktop::find_desktop_apps();
        let mut resolved_icon = None;
        for app in apps {
            if app.name.to_lowercase() == lower_name {
                if let Some(ref app_icon) = app.icon {
                    resolved_icon = Some(app_icon.clone());
                }
                break;
            }
        }
        
        if let Some(icon_name) = resolved_icon {
            let mut clean_resolved = icon_name;
            for ext in &[".png", ".svg", ".xpm", ".jpg", ".jpeg", ".gif"] {
                if clean_resolved.to_lowercase().ends_with(ext) {
                    clean_resolved = clean_resolved[..clean_resolved.len() - ext.len()].to_string();
                    break;
                }
            }
            gtk4::Image::from_icon_name(&clean_resolved)
        } else if let Some(ref disp) = display {
            let theme = gtk4::IconTheme::for_display(disp);
            if theme.has_icon(default_fallback) {
                gtk4::Image::from_icon_name(default_fallback)
            } else {
                gtk4::Image::from_icon_name("image-missing")
            }
        } else {
            gtk4::Image::from_icon_name("image-missing")
        }
    }
}
