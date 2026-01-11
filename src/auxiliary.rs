//!
//! Auxiliary functions
//!

use anyhow::{anyhow, bail, Result};
use std::{ffi::OsStr, path::Path};

/// Check if 16-bit start address can contain `length` bytes
///
/// # Examples
/// ```
/// use ultimate64::auxiliary::check_address_overflow;
/// assert!(check_address_overflow(0xffff, 1).is_ok());
/// assert!(check_address_overflow(0xffff, 2).is_err());
/// ```
///
pub fn check_address_overflow(address: u16, length: u16) -> Result<()> {
    if length > 0 && u16::checked_add(address, length - 1).is_none() {
        bail!(
            "Address {:#06x} + length {:#06x} overflows address space",
            address,
            length
        )
    } else {
        Ok(())
    }
}

/// Helper function to extract file extension from `path` to a lowercase string.
/// Returns `None` if `path` has no extension.
///
/// # Examples
/// ```
/// use ultimate64::auxiliary::get_extension;
/// let path = std::ffi::OsString::from("foo.bAR");
/// let ext = get_extension(&path).unwrap();
/// assert_eq!(ext, "bar");
/// ```
pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
}

/// Helper funtion to extract load address from first two bytes of data, little endian format
///
/// # Examples
/// ```
/// use ultimate64::auxiliary::extract_load_address;
/// let data = vec![0x01, 0x08, 0x00, 0x00];
/// let addr = extract_load_address(&data).unwrap();
/// assert_eq!(addr, 0x0801);
/// let data = vec![0x01];
/// assert!(extract_load_address(&data).is_err());
/// ```
pub fn extract_load_address(data: &[u8]) -> Result<u16> {
    data.get(..2)
        .ok_or_else(|| anyhow!("at least two bytes required to detect load address"))
        .map(|b| b.try_into().unwrap()) // -> [u8; 2] -  panic impossible
        .map(u16::from_le_bytes) // -> u16 using little-endian byte order
}
