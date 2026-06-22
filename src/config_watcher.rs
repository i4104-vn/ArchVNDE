use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, EventKind};
use crate::config::Config;

/// Reads and parses the TOML config at `path`.
///
/// If the file does not exist it is created with default values.
/// Falls back to [`Config::default`] on any parse or I/O error.
pub fn load_config(path: &Path) -> Config {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let default = Config::default();
        if let Ok(s) = toml::to_string_pretty(&default) {
            let _ = std::fs::write(path, s);
        }
        return default;
    }

    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => {
                tracing::info!("config loaded from {:?}", path);
                config
            }
            Err(err) => {
                tracing::error!("TOML parse error in {:?}: {}", path, err);
                Config::default()
            }
        },
        Err(err) => {
            tracing::error!("cannot read config {:?}: {}", path, err);
            Config::default()
        }
    }
}

/// Spawns a filesystem watcher that hot-reloads the config on every file save.
///
/// The returned [`RecommendedWatcher`] must be kept alive for the duration of
/// the compositor; dropping it stops the watcher.
pub fn spawn_config_watcher(path: PathBuf, config: Arc<ArcSwap<Config>>) -> RecommendedWatcher {
    let path_clone = path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    std::thread::sleep(Duration::from_millis(100));
                    let new = load_config(&path_clone);
                    config.store(Arc::new(new));
                    tracing::info!("config reloaded");
                }
            }
        },
        notify::Config::default(),
    )
    .expect("failed to create config watcher");

    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .expect("failed to watch config path");

    watcher
}
