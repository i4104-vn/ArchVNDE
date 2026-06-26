use serde::{Deserialize, Serialize};
use super::pinned_app::PinnedApp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockConfig {
    pub pinned_apps: Vec<PinnedApp>,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            pinned_apps: vec![
                PinnedApp {
                    name: "Terminal".to_string(),
                    icon: "terminal".to_string(),
                    command: "foot".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "Files".to_string(),
                    icon: "folder".to_string(),
                    command: "pcmanfm".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "Web Browser".to_string(),
                    icon: "search".to_string(),
                    command: "firefox".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "Music Player".to_string(),
                    icon: "music".to_string(),
                    command: "amberol".to_string(),
                    args: vec![],
                },
                PinnedApp {
                    name: "System Settings".to_string(),
                    icon: "settings".to_string(),
                    command: "gnome-control-center".to_string(),
                    args: vec![],
                },
            ],
        }
    }
}
