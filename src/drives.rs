//! # Disk drive and disk image manipulation

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Disk drive types
#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum DriveType {
    #[serde(rename = "1541")]
    CBM1541,
    #[serde(rename = "1571")]
    CBM1571,
    #[serde(rename = "1581")]
    CBM1581,
}

/// Disk image types
#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiskImageType {
    #[clap(name = "d64")]
    D64,
    #[clap(name = "g64")]
    G64,
    #[clap(name = "d71")]
    D71,
    #[clap(name = "g71")]
    G71,
    #[clap(name = "d81")]
    D81,
}

impl DiskImageType {
    /// Helper function to extract file extension from `file` to a lowercase string
    fn get_extension(file: &std::ffi::OsString) -> String {
        std::path::Path::new(&file)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default()
            .to_lowercase()
    }

    /// New disk image type from file name
    pub fn from_file_name(path: &std::ffi::OsString) -> Result<Self> {
        match Self::get_extension(path).as_str() {
            "d64" => Ok(DiskImageType::D64),
            "g64" => Ok(DiskImageType::G64),
            "d71" => Ok(DiskImageType::D71),
            "g71" => Ok(DiskImageType::G71),
            "d81" => Ok(DiskImageType::D81),
            _ => Err(anyhow::anyhow!(
                "File extension must be one of: d64, d71, d81, g64, g71"
            )),
        }
    }
    /// Get file extension for disk image type
    pub fn extension(&self) -> String {
        match self {
            DiskImageType::D64 => "d64".to_string(),
            DiskImageType::G64 => "g64".to_string(),
            DiskImageType::D71 => "d71".to_string(),
            DiskImageType::G71 => "g71".to_string(),
            DiskImageType::D81 => "d81".to_string(),
        }
    }
}

impl TryFrom<&str> for DiskImageType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "d64" => Ok(DiskImageType::D64),
            "g64" => Ok(DiskImageType::G64),
            "d71" => Ok(DiskImageType::D71),
            "g71" => Ok(DiskImageType::G71),
            "d81" => Ok(DiskImageType::D81),
            _ => Err(format!("Unknown disk image type: {}", s)),
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
#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum MountMode {
    /// Read and write access
    #[clap(name = "rw")]
    ReadWrite,
    /// Read only access
    #[clap(name = "ro")]
    ReadOnly,
    /// Unlinked
    #[clap(name = "unlinked")]
    Unlinked,
}

impl TryFrom<&str> for MountMode {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "rw" => Ok(MountMode::ReadWrite),
            "ro" => Ok(MountMode::ReadOnly),
            "unlinked" => Ok(MountMode::Unlinked),
            _ => Err(format!("Unknown mount mode: {}", s)),
        }
    }
}

impl From<MountMode> for String {
    fn from(m: MountMode) -> Self {
        match m {
            MountMode::ReadWrite => "rw".to_string(),
            MountMode::ReadOnly => "ro".to_string(),
            MountMode::Unlinked => "unlinked".to_string(),
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
