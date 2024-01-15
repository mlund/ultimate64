//! # Disk drive and disk image manipulation

use serde::{Deserialize, Serialize};

/// Disk drive types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DriveType {
    #[serde(rename = "1541")]
    CBM1541,
    #[serde(rename = "1571")]
    CBM1571,
    #[serde(rename = "1581")]
    CBM1581,
}

/// Disk image types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DiskImageType {
    #[serde(rename = "d64")]
    D64,
    #[serde(rename = "g64")]
    G64,
    #[serde(rename = "d71")]
    D71,
    #[serde(rename = "g71")]
    G71,
    #[serde(rename = "d81")]
    D81,
}

impl From<&str> for DiskImageType {
    fn from(s: &str) -> Self {
        match s {
            "d64" => DiskImageType::D64,
            "g64" => DiskImageType::G64,
            "d71" => DiskImageType::D71,
            "g71" => DiskImageType::G71,
            "d81" => DiskImageType::D81,
            _ => panic!("Unknown disk image type: {}", s),
        }
    }
}

impl From<DiskImageType> for String {
    fn from(d: DiskImageType) -> Self {
        match d {
            DiskImageType::D64 => "d64".to_string(),
            DiskImageType::G64 => "g64".to_string(),
            DiskImageType::D71 => "d71".to_string(),
            DiskImageType::G71 => "g71".to_string(),
            DiskImageType::D81 => "d81".to_string(),
        }
    }
}

/// Drive mount modes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MountMode {
    ReadWrite,
    ReadOnly,
    Unlinked,
}

impl From<&str> for MountMode {
    fn from(s: &str) -> Self {
        match s {
            "rw" => MountMode::ReadWrite,
            "ro" => MountMode::ReadOnly,
            "ul" => MountMode::Unlinked,
            _ => panic!("Unknown mount mode: {}", s),
        }
    }
}

impl From<MountMode> for String {
    fn from(m: MountMode) -> Self {
        match m {
            MountMode::ReadWrite => "rw".to_string(),
            MountMode::ReadOnly => "ro".to_string(),
            MountMode::Unlinked => "ul".to_string(),
        }
    }
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
    pub drive_type: Option<DriveType>,
    /// Last error
    pub last_error: Option<String>,
    /// ROM
    pub rom: Option<String>,
    /// Image file
    pub image_file: Option<String>,
    /// Image path
    pub image_path: Option<String>,
}
