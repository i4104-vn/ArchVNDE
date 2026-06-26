use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IslandState {
    pub active: bool,
    pub title: String,
    pub subtitle: String,
    pub icon: String,
}
