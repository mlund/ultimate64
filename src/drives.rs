//! # Disk drive and disk image manipulation

use crate::aux;
use anyhow::{bail, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

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
    /// New disk image type from file name
    pub fn from_file_name<T: AsRef<Path>>(path: T) -> Result<Self> {
        let ext = aux::get_extension(path).unwrap_or_default();
        match ext.as_str() {
            "d64" => Ok(Self::D64),
            "g64" => Ok(Self::G64),
            "d71" => Ok(Self::D71),
            "g71" => Ok(Self::G71),
            "d81" => Ok(Self::D81),
            _ => bail!("File extension must be one of: d64, d71, d81, g64, g71"),
        }
    }
}

impl Display for DiskImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DiskImageType::D64 => "d64",
            DiskImageType::G64 => "g64",
            DiskImageType::D71 => "d71",
            DiskImageType::G71 => "g71",
            DiskImageType::D81 => "d81",
        };
        write!(f, "{s}")
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
        use MountMode::*;
        match s {
            "readwrite" => Ok(ReadWrite),
            "readonly" => Ok(ReadOnly),
            "unlinked" => Ok(Unlinked),
            _ => Err(format!("Unknown mount mode: {s}")),
        }
    }
}

impl Display for MountMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MountMode::*;
        let s = match self {
            ReadWrite => "readwrite",
            ReadOnly => "readonly",
            Unlinked => "unlinked",
        };
        write!(f, "{s}")
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
