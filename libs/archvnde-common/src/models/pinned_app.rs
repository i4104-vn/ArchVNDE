use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinnedApp {
    pub name: String,
    pub icon: String,
    pub command: String,
    pub args: Vec<String>,
}
