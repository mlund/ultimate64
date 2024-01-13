use anyhow::Result;
use serde::{Deserialize, Serialize};

pub enum DriveType {
    Floppy1541,
    Floppy1571,
    Floppy1581,
    Unknown,
}

/// Drive description
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Drive {
    /// Bus ID
    pub bus_id: u8,
    /// Enabled
    pub enabled: bool,
    /// Type
    #[serde(rename = "type")]
    pub drive_type: Option<String>,
    /// Last error
    pub last_error: Option<String>,
    /// ROM
    pub rom: Option<String>,
    /// Image file
    pub image_file: Option<String>,
    /// Image path
    pub image_path: Option<String>,
}
