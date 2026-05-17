// FILE: crates/gargantua-video/src/config.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    pub width:  u32,
    pub height: u32,
    pub fps:    u32,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            width:  1920,
            height: 1080,
            fps:    60,
        }
    }
}
