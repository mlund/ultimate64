//!
//! Auxiliary functions
//!

use anyhow::Result;
use log::debug;

/// Check if 16-bit start address can contain `length` bytes
///
/// # Examples
/// ```
/// use ultimate64::aux::check_address_overflow;
/// assert!(check_address_overflow(0xffff, 1).is_ok());
/// assert!(check_address_overflow(0xffff, 2).is_err());
/// ```
///
pub fn check_address_overflow(address: u16, length: u16) -> Result<()> {
    if length > 0 && u16::checked_add(address, length - 1).is_none() {
        Err(anyhow::anyhow!(
            "Address {:#06x} + length {:#06x} overflows address space",
            address,
            length
        ))
    } else {
        Ok(())
    }
}

/// Helper function to extract file extension from `path` to a lowercase string.
/// Returns `None` if `path` has no extension.
///
/// # Examples
/// ```
/// use ultimate64::aux::get_extension;
/// let path = std::ffi::OsString::from("foo.bAR");
/// let ext = get_extension(&path).unwrap();
/// assert_eq!(ext, "bar");
/// ```
pub fn get_extension(path: &std::ffi::OsString) -> Option<String> {
    std::path::Path::new(&path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|s| s.to_lowercase())
}

/// Helper funtion to extract load address from first two bytes of data, little endian format
///
/// # Examples
/// ```
/// use ultimate64::aux::extract_load_address;
/// let data = vec![0x01, 0x08, 0x00, 0x00];
/// let load_address = extract_load_address(&data).unwrap();
/// assert_eq!(load_address, 0x0801);
/// ```
pub fn extract_load_address(data: &[u8]) -> Result<u16> {
    if data.len() < 2 {
        Err(anyhow::anyhow!(
            "Data must be two or more bytes to detect load address"
        ))
    } else {
        let load_address = u16::from_le_bytes([data[0], data[1]]);
        debug!("Detected load address: {:#06x}", load_address);
        Ok(load_address)
    }
}
