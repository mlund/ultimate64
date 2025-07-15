//!
//! Rust library and command line interface for interfacing with [Ultimate-64 and Ultimate-II](https://ultimate64.com)
//! hardware using the
//! [REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html).
//!

use crate::{
    drives::{DiskImageType, Drive, DriveList},
    petscii::Petscii,
};
use anyhow::{anyhow, bail, Ok, Result};
use log::{debug, warn};
use reqwest::blocking::Client;
use std::{collections::HashMap, path::Path, thread::sleep, time::Duration};
use url::Host;

pub mod aux;
pub mod drives;
pub mod petscii;

/// Communication with Ultimate series using
/// the [REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html)
///
/// # Examples
/// ~~~ rust, ignore
/// use ultimate64::Rest;
/// let ultimate = Rest::new("192.168.1.10");
/// ultimate.reset();
/// ~~~
#[derive(Debug)]
pub struct Rest {
    /// HTTP client
    client: Client,
    /// Header
    url_pfx: String,
}

impl Rest {
    /// Create new Rest instance
    ///
    /// # Arguments
    ///
    /// * `host` - Hostname or IP address of Ultimate-64 of Ultimate-II
    pub fn new(host: &Host) -> Self {
        Self {
            client: Client::new(),
            url_pfx: format!("http://{host}/v1"),
        }
    }

    fn put(&self, path: &str) -> Result<()> {
        let url = format!("{}/{}", self.url_pfx, path);
        self.client.put(url).send()?;
        Ok(())
    }

    /// Get version
    pub fn version(&self) -> Result<String> {
        let url = format!("{}/version", self.url_pfx);
        let response = self.client.get(url).send()?;
        let body = response.text()?;
        Ok(body)
    }

    /// Get drives
    pub fn drives(&self) -> Result<String> {
        let url = format!("{}/drives", self.url_pfx);
        let response = self.client.get(url).send()?;
        let body = response.text()?;
        Ok(body)
    }

    /// Load PRG bytes into memory - do NOT run.
    /// The machine resets, and loads the attached program into memory using DMA.
    pub fn load_prg(&self, prg_data: &[u8]) -> Result<()> {
        debug!("Load PRG file of {} bytes", prg_data.len());
        let url = format!("{}/runners:load_prg", self.url_pfx);
        self.client.post(url).body(prg_data.to_vec()).send()?;
        Ok(())
    }

    /// Load and run PRG bytes into memory
    ///
    /// The machine resets, and loads the attached program into memory using DMA.
    pub fn run_prg(&self, data: &[u8]) -> Result<()> {
        debug!("Run PRG file of {} bytes", data.len());
        let url = format!("{}/runners:run_prg", self.url_pfx);
        self.client.post(url).body(data.to_vec()).send()?;
        Ok(())
    }

    /// Start supplied cartridge file
    ///
    /// The ‘crt’ file is attached to the POST request.
    /// The machine resets, with the attached cartridge active.
    /// It does not alter the configuration of the Ultimate.
    pub fn run_crt(&self, data: &[u8]) -> Result<()> {
        debug!("Run CRT file of {} bytes", data.len());
        let url = format!("{}/runners:run_crt", self.url_pfx);
        self.client.post(url).body(data.to_vec()).send()?;
        Ok(())
    }

    /// Reset machine
    pub fn reset(&self) -> Result<()> {
        debug!("Reset machine");
        self.put("machine:reset")?;
        Ok(())
    }
    /// Reboot machine
    pub fn reboot(&self) -> Result<()> {
        debug!("Reboot machine");
        self.put("machine:reboot")?;
        Ok(())
    }

    /// Pause machine
    pub fn pause(&self) -> Result<()> {
        debug!("Pause machine");
        self.put("machine:pause")?;
        Ok(())
    }

    /// Resume machine
    pub fn resume(&self) -> Result<()> {
        debug!("Resume machine");
        self.put("machine:resume")?;
        Ok(())
    }
    /// Poweroff machine
    pub fn poweroff(&self) -> Result<()> {
        debug!("Poweroff machine");
        self.put("machine:poweroff")?;
        Ok(())
    }

    /// Write data to memory using a POST request
    pub fn write_mem(&self, address: u16, data: &[u8]) -> Result<()> {
        aux::check_address_overflow(address, data.len() as u16)?;
        if matches!(address, 0 | 1) {
            warn!("Warning: DMA cannot access internal CPU registers at address 0 and 1");
        }
        let url = format!("{}/machine:writemem?address={:x}", self.url_pfx, address);
        self.client.post(url).body(data.to_vec()).send()?;
        debug!("Wrote {} byte(s) to {:#06x}", data.len(), address);
        Ok(())
    }

    /// Emulate keyboard input
    pub fn type_text(&self, s: &str) -> Result<()> {
        debug!("Emulating keyboard typing: {s}");
        const TAIL_PTR: u16 = 0x00c5;
        const HEAD_PTR: u16 = 0x00c6;
        const BUFFER_BASE: u16 = 0x0277;

        // the C64 input buffer is limited to 10 characters
        for chunk in s.chars().collect::<Vec<_>>().chunks(10) {
            self.write_mem(TAIL_PTR, &[0, 0])?; // clear keyboard buffer
            chunk.iter().enumerate().try_for_each(|(i, c)| {
                let byte = Petscii::from_str_lossy(&c.to_string())[0];
                self.write_mem(BUFFER_BASE + i as u16, &[byte])
            })?;
            self.write_mem(HEAD_PTR, &[chunk.len() as u8])?;
            sleep(Duration::from_millis(50)); // wait for C64 to process input
        }
        Ok(())
    }

    /// Read `length` bytes from `address`
    pub fn read_mem(&self, address: u16, length: u16) -> Result<Vec<u8>> {
        aux::check_address_overflow(address, length)?;
        if matches!(address, 0 | 1) {
            warn!("Warning: DMA cannot access internal CPU registers at address 0 and 1");
        }
        let url = format!(
            "{}/machine:readmem?address={:x}&length={}",
            self.url_pfx, address, length
        );
        let bytes = self.client.get(url).send()?.bytes()?.to_vec();
        debug!("Read {length} byte(s) from {address:#06x}");
        Ok(bytes)
    }

    /// Play SID file
    pub fn sid_play(&self, siddata: &[u8], songnr: Option<u8>) -> Result<()> {
        let url = match songnr {
            Some(songnr) => format!("{}/runners:sidplay?songnr={}", self.url_pfx, songnr),
            None => format!("{}/runners:sidplay", self.url_pfx),
        };
        self.client.post(url).body(siddata.to_vec()).send()?;
        Ok(())
    }

    /// Play amiga MOD file
    pub fn mod_play(&self, moddata: &[u8]) -> Result<()> {
        let url = format!("{}/runners:modplay", self.url_pfx);
        self.client.post(url).body(moddata.to_vec()).send()?;
        Ok(())
    }

    /// Load data into memory using either a custom address, or deduce the
    /// load address from the first two bytes of the data (little endian).
    /// In the case of the latter, the first two bytes are not written to memory.
    pub fn load_data(&self, data: &[u8], address: Option<u16>) -> Result<()> {
        match address {
            Some(address) => self.write_mem(address, data),
            None => {
                let load_address = aux::extract_load_address(data)?;
                self.write_mem(load_address, &data[2..]) // skip first two bytes
            }
        }
    }

    /// Get drive list
    pub fn drive_list(&self) -> Result<HashMap<String, Drive>> {
        let url = format!("{}/drives", self.url_pfx);
        let response = self.client.get(url).send()?;
        let nested: DriveList = response.json()?;
        let drives = nested
            .drives
            .iter()
            .flat_map(|m| m.iter().map(|(name, drive)| (name.clone(), drive.clone())))
            .collect();
        Ok(drives)
    }

    /// Mount disk image
    ///
    /// Curl equivalent:
    /// `curl -X POST 192.168.68.81/v1/drives/a:mount -F "file=@disk.d64" -F "mode=readwrite" -F "type=d64"`
    pub fn mount_disk_image<P: AsRef<Path>>(
        &self,
        path: P,
        drive: String,
        mount_mode: drives::MountMode,
        run: bool,
    ) -> Result<()> {
        let disktype = DiskImageType::from_file_name(&path)?;
        let url = format!("{}/drives/{drive}:mount", self.url_pfx);
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", path)
            .map_err(|e| anyhow!("disk image error: {e}"))?
            .text("mode", mount_mode.to_string())
            .text("type", disktype.to_string());
        let response = self.client.post(url).multipart(form).send()?;
        if response.status().is_client_error() {
            bail!(
                "disk mount error: {} - {}",
                response.status(),
                response.text().unwrap()
            );
        }
        if run {
            self.reset()?;
            sleep(Duration::from_millis(2000));
            self.type_text("load\"*\",8,1\nrun\n")?;
        }
        Ok(())
    }
}
