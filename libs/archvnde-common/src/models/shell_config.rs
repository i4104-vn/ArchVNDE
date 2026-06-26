use serde::{Deserialize, Serialize};
use super::theme_config::ThemeConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellConfig {
    pub theme: ThemeConfig,
}
