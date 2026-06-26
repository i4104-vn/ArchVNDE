use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub blur_radius: u32,
    pub opacity: f64,
    pub border_color: String,
    pub border_width: u32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            blur_radius: 20,
            opacity: 0.75,
            border_color: "#ffffff".to_string(),
            border_width: 1,
        }
    }
}
