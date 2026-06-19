use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, EventKind};
use crate::config::Config;

pub fn load_config(path: &Path) -> Config {
    if !path.exists() {
        // Tạo file cấu hình mặc định nếu chưa tồn tại
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let default_config = Config::default();
        if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
            let _ = std::fs::write(path, toml_str);
        }
        return default_config;
    }

    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => {
                tracing::info!("Cấu hình loaded thành công từ {:?}", path);
                config
            }
            Err(err) => {
                tracing::error!("Lỗi cú pháp TOML trong {:?}: {}. Sử dụng cấu hình mặc định.", path, err);
                Config::default()
            }
        },
        Err(err) => {
            tracing::error!("Không thể đọc tệp cấu hình {:?}: {}. Sử dụng cấu hình mặc định.", path, err);
            Config::default()
        }
    }
}

pub fn spawn_config_watcher(path: PathBuf, shared_config: Arc<ArcSwap<Config>>) -> RecommendedWatcher {
    let path_clone = path.clone();
    let config_clone = shared_config.clone();

    // Tạo watcher từ notify crate
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                // Chỉ xử lý các sự kiện ghi đè/lưu file (Modify)
                if matches!(event.kind, EventKind::Modify(_)) {
                    // Debounce đơn giản: đợi file lưu xong hẳn
                    std::thread::sleep(Duration::from_millis(100));
                    
                    tracing::info!("Phát hiện thay đổi cấu hình. Đang tải lại...");
                    let new_config = load_config(&path_clone);
                    config_clone.store(Arc::new(new_config));
                    tracing::info!("Đã tải lại cấu hình hệ thống thành công!");
                }
            }
        },
        notify::Config::default(),
    ).expect("Không thể tạo trình theo dõi file Config");

    watcher.watch(&path, RecursiveMode::NonRecursive).expect("Lỗi thiết lập watcher");
    
    watcher
}
