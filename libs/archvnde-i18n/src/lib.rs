use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct Config {
    #[serde(default = "default_locale")]
    locale: String,
}

fn default_locale() -> String {
    "vi".to_string()
}

static CURRENT_LOCALE: OnceLock<RwLock<String>> = OnceLock::new();

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
        .join("archvnde")
        .join("config.toml")
}

fn load_locale_from_disk() -> String {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                if config.locale == "en" || config.locale == "vi" {
                    return config.locale;
                }
            }
        }
    }
    default_locale()
}

pub fn get_locale() -> String {
    let lock = CURRENT_LOCALE.get_or_init(|| {
        RwLock::new(load_locale_from_disk())
    });
    lock.read().unwrap().clone()
}

pub fn set_locale(locale: &str) {
    let normalized = if locale == "en" { "en" } else { "vi" };
    
    // Update memory cache
    let lock = CURRENT_LOCALE.get_or_init(|| {
        RwLock::new(normalized.to_string())
    });
    if let Ok(mut writer) = lock.write() {
        *writer = normalized.to_string();
    }

    // Persist to ~/.config/archvnde/config.toml
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Read existing toml first to preserve other keys if any
    let mut config_map = if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            toml::from_str::<toml::Table>(&content).unwrap_or_default()
        } else {
            toml::Table::new()
        }
    } else {
        toml::Table::new()
    };

    config_map.insert("locale".to_string(), toml::Value::String(normalized.to_string()));

    if let Ok(serialized) = toml::to_string(&config_map) {
        let _ = fs::write(path, serialized);
    }
}

pub fn t(key: &str) -> String {
    let locale = get_locale();
    let dict = get_translations(&locale);
    dict.get(key)
        .map(|&s| s.to_string())
        .unwrap_or_else(|| key.to_string())
}

fn get_translations(locale: &str) -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    match locale {
        "en" => {
            // Menu
            map.insert("menu.terminal", "Terminal");
            map.insert("menu.file_manager", "File Manager");
            map.insert("menu.change_wallpaper", "Change Wallpaper");
            map.insert("menu.reconfigure_shell", "Reconfigure Shell");
            map.insert("menu.exit_shell", "Exit Shell");
            
            // Launcher
            map.insert("launcher.search_placeholder", "Search apps or files...");
            map.insert("launcher.welcome", "Type keywords to search apps and files...");
            map.insert("launcher.apps", "Applications");
            map.insert("launcher.files", "Files");
            map.insert("launcher.no_results", "No matching results found");
            map.insert("launcher.google_search", "Search Google for \"{}\"");

            // Panel / Clock
            map.insert("panel.no_notifications", "No new notifications");
            map.insert("panel.notifications", "Notifications");
            map.insert("panel.storage_usage", "Storage Usage");
            map.insert("panel.no_storage", "No physical storage found");
            map.insert("panel.system", "System");
            map.insert("panel.clear_all", "Clear All");
            map.insert("panel.just_now", "Just now");
            map.insert("panel.minutes_ago", "{}m ago");
            map.insert("panel.hours_ago", "{}h ago");
            map.insert("panel.days_ago", "{}d ago");

            // Control center
            map.insert("control.network", "Network");
            map.insert("control.connected", "Connected");
            map.insert("control.bluetooth", "Bluetooth");
            map.insert("control.not_connected", "Not Connected");
            map.insert("control.dnd", "Do Not Disturb");
            map.insert("control.on", "On");
            map.insert("control.off", "Off");
            map.insert("control.dark_mode", "Dark\nMode");
            map.insert("control.night_light", "Night\nColor");
            map.insert("control.title", "Control Center");
            map.insert("control.lang_changed_title", "Language Changed");
            map.insert("control.lang_changed_msg", "Restart widgets to apply changes system-wide.");

            // Screenshot
            map.insert("screenshot.reset_tooltip", "Discard and restart (Clear all drawings)");
            map.insert("screenshot.pen_tooltip", "Pen");
            map.insert("screenshot.rect_tooltip", "Draw rectangle");
            map.insert("screenshot.blur_tooltip", "Blur information");
            map.insert("screenshot.eraser_tooltip", "Erase drawings");
            map.insert("screenshot.color_tooltip", "Select drawing color");
            map.insert("screenshot.copy_tooltip", "Copy to Clipboard (Enter)");
            map.insert("screenshot.save_tooltip", "Save screenshot (Ctrl+S)");
            map.insert("screenshot.cancel_tooltip", "Cancel (Escape)");
            map.insert("screenshot.copied_title", "Copied Screenshot");
            map.insert("screenshot.copied_msg", "Screenshot has been saved to your clipboard.");
            map.insert("screenshot.saved_title", "Screenshot Saved");
            map.insert("screenshot.saved_msg", "Saved to {}");
            map.insert("screenshot.full_saved_title", "Fullscreen Captured");

            // Colors
            map.insert("color.red", "Red");
            map.insert("color.orange", "Orange");
            map.insert("color.yellow", "Yellow");
            map.insert("color.green", "Green");
            map.insert("color.blue", "Blue");
            map.insert("color.purple", "Purple");
            map.insert("color.white", "White");
            map.insert("color.black", "Black");
        }
        _ => { // vi is default
            // Menu
            map.insert("menu.terminal", "Terminal");
            map.insert("menu.file_manager", "Trình quản lý tệp");
            map.insert("menu.change_wallpaper", "Thay đổi hình nền");
            map.insert("menu.reconfigure_shell", "Cấu hình lại Shell");
            map.insert("menu.exit_shell", "Thoát Shell");

            // Launcher
            map.insert("launcher.search_placeholder", "Tìm ứng dụng hoặc tệp tin...");
            map.insert("launcher.welcome", "Nhập từ khóa để tìm kiếm ứng dụng và tệp tin...");
            map.insert("launcher.apps", "Ứng dụng");
            map.insert("launcher.files", "Tập tin");
            map.insert("launcher.no_results", "Không tìm thấy kết quả phù hợp");
            map.insert("launcher.google_search", "Tìm trên Google cho \"{}\"");

            // Panel / Clock
            map.insert("panel.no_notifications", "Không có thông báo mới");
            map.insert("panel.notifications", "Thông báo");
            map.insert("panel.storage_usage", "Dung lượng đĩa");
            map.insert("panel.no_storage", "Không tìm thấy ổ lưu trữ");
            map.insert("panel.system", "Hệ thống");
            map.insert("panel.clear_all", "Xóa tất cả");
            map.insert("panel.just_now", "Vừa xong");
            map.insert("panel.minutes_ago", "{} phút trước");
            map.insert("panel.hours_ago", "{} giờ trước");
            map.insert("panel.days_ago", "{} ngày trước");

            // Control center
            map.insert("control.network", "Mạng");
            map.insert("control.connected", "Đã kết nối");
            map.insert("control.bluetooth", "Bluetooth");
            map.insert("control.not_connected", "Chưa kết nối");
            map.insert("control.dnd", "Chế độ không làm phiền");
            map.insert("control.on", "Bật");
            map.insert("control.off", "Tắt");
            map.insert("control.dark_mode", "Chế độ\nTối");
            map.insert("control.night_light", "Ánh sáng\nĐêm");
            map.insert("control.title", "Trung tâm Điều khiển");
            map.insert("control.lang_changed_title", "Đã thay đổi ngôn ngữ");
            map.insert("control.lang_changed_msg", "Khởi động lại widgets để áp dụng toàn hệ thống.");

            // Screenshot
            map.insert("screenshot.reset_tooltip", "Bỏ chụp và làm lại (Xóa hết nét vẽ)");
            map.insert("screenshot.pen_tooltip", "Bút vẽ");
            map.insert("screenshot.rect_tooltip", "Vẽ hình chữ nhật");
            map.insert("screenshot.blur_tooltip", "Làm mờ thông tin");
            map.insert("screenshot.eraser_tooltip", "Xóa hình vẽ");
            map.insert("screenshot.color_tooltip", "Chọn màu vẽ");
            map.insert("screenshot.copy_tooltip", "Sao chép vào Clipboard (Enter)");
            map.insert("screenshot.save_tooltip", "Lưu ảnh chụp (Ctrl+S)");
            map.insert("screenshot.cancel_tooltip", "Hủy (Escape)");
            map.insert("screenshot.copied_title", "Đã sao chép ảnh");
            map.insert("screenshot.copied_msg", "Ảnh chụp đã được lưu vào clipboard.");
            map.insert("screenshot.saved_title", "Đã chụp ảnh màn hình");
            map.insert("screenshot.saved_msg", "Đã lưu tại {}");
            map.insert("screenshot.full_saved_title", "Đã chụp toàn màn hình");

            // Colors
            map.insert("color.red", "Đỏ");
            map.insert("color.orange", "Cam");
            map.insert("color.yellow", "Vàng");
            map.insert("color.green", "Lục");
            map.insert("color.blue", "Lam");
            map.insert("color.purple", "Tím");
            map.insert("color.white", "Trắng");
            map.insert("color.black", "Đen");
        }
    }
    map
}
