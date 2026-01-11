//!
//! Rust library and command line interface for interfacing with [Ultimate-64 and Ultimate-II](https://ultimate64.com)
//! hardware using the
//! [REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html).
//!

use crate::{
    auxiliary::check_address_overflow,
    drives::{DiskImageType, Drive, DriveList},
    petscii::Petscii,
};
use anyhow::{anyhow, bail, ensure, Ok, Result};
use clap::ValueEnum;
use core::fmt::Display;
use log::{debug, warn};
use reqwest::{
    blocking::{Body, Client, Response},
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use std::{collections::HashMap, path::Path, thread::sleep, time::Duration};
use url::Host;

pub mod auxiliary;
pub mod drives;
pub mod petscii;
pub mod vicstream;

/// Ultimate-64 and Ultimate-II device information
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Product name
    pub product: String,
    /// Firmware version
    pub firmware_version: String,
    /// FPGA version
    pub fpga_version: String,
    /// Core version (only for Ultimate-64)
    pub core_version: Option<String>,
    /// Hostname
    pub hostname: String,
    /// Unique ID (unless disabled under "Network Settings")
    pub unique_id: Option<String>,
}

impl Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (firmware {}, fpga {}, core {}, id {})",
            self.product,
            self.firmware_version,
            self.fpga_version,
            self.core_version.as_deref().unwrap_or("N/A"),
            self.unique_id.as_deref().unwrap_or("N/A")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StreamType {
    /// Video stream
    Video,
    /// Audio stream
    Audio,
    /// Debug stream
    Debug,
}

impl StreamType {
    /// Default port for the stream type (video = 11000, audio = 11001, debug = 11002)
    pub fn default_port(&self) -> u16 {
        match self {
            StreamType::Video => 11000,
            StreamType::Audio => 11001,
            StreamType::Debug => 11002,
        }
    }
}

impl Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StreamType::*;
        match self {
            Video => write!(f, "video"),
            Audio => write!(f, "audio"),
            Debug => write!(f, "debug"),
        }
    }
}

/// Communication with Ultimate series using
/// the [REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html)
///
/// # Examples
/// ~~~ rust, ignore
/// use ultimate64::Rest;
/// let ultimate = Rest::new("192.168.1.10", None).unwrap();
/// ultimate.reset();
/// ~~~
#[derive(Debug)]
pub struct Rest {
    /// HTTP client
    client: Client,
    /// Header
    url_prefix: String,
    /// Headers
    headers: HeaderMap,
}

impl Rest {
    /// Create new Rest instance
    pub fn new(host: &Host, password: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::default();
        if let Some(pw) = password {
            headers.insert("X-password", HeaderValue::from_str(pw.as_str())?);
        }

        Ok(Self {
            client: Client::new(),
            url_prefix: format!("http://{host}/v1"),
            headers,
        })
    }

    /// Check sanity of response
    fn check_response(response: &Response) -> Result<()> {
        // Handle a few specific status codes
        match response.status() {
            StatusCode::FORBIDDEN => bail!("access denied: check password or device settings"),
            StatusCode::NOT_IMPLEMENTED => bail!("command unavailable on this Ultimate device"),
            _ => {}
        }
        ensure!(
            response.status().is_success(),
            "request failed with status: {}",
            response.status()
        );
        Ok(())
    }

    /// HTTP PUT request
    fn put(&self, path: &str) -> Result<Response> {
        let url = format!("{}/{}", self.url_prefix, path);
        let response = self.client.put(url).headers(self.headers.clone()).send()?;
        Self::check_response(&response)?;
        Ok(response)
    }

    /// HTTP GET request
    fn get(&self, path: &str) -> Result<Response> {
        let url = format!("{}/{}", self.url_prefix, path);
        let response = self.client.get(url).headers(self.headers.clone()).send()?;
        Self::check_response(&response)?;
        Ok(response)
    }

    /// HTTP POST request with body
    fn post<T: Into<Body>>(&self, path: &str, body: T) -> Result<Response> {
        let url = format!("{}/{}", self.url_prefix, path);
        let response = self
            .client
            .post(url)
            .body(body)
            .headers(self.headers.clone())
            .send()?;
        Self::check_response(&response)?;
        Ok(response)
    }

    /// Get device information
    pub fn info(&self) -> Result<DeviceInfo> {
        let body = self.get("info")?.text()?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Get version
    pub fn version(&self) -> Result<String> {
        let response = self.get("version")?;
        let body = response.text()?;
        Ok(body)
    }

    /// Get drives
    pub fn drives(&self) -> Result<String> {
        let response = self.get("drives")?;
        let body = response.text()?;
        Ok(body)
    }

    /// Load PRG bytes into memory - do NOT run.
    /// The machine resets, and loads the attached program into memory using DMA.
    pub fn load_prg(&self, prg_data: &[u8]) -> Result<()> {
        debug!("Load PRG file of {} bytes", prg_data.len());
        self.post("runners:load_prg", prg_data.to_vec())?;
        Ok(())
    }

    /// Load and run PRG bytes into memory
    ///
    /// The machine resets, and loads the attached program into memory using DMA.
    pub fn run_prg(&self, data: &[u8]) -> Result<()> {
        debug!("Run PRG file of {} bytes", data.len());
        self.post("runners:run_prg", data.to_vec())?;
        Ok(())
    }

    /// Start supplied cartridge file
    ///
    /// The ‘crt’ file is attached to the POST request.
    /// The machine resets, with the attached cartridge active.
    /// It does not alter the configuration of the Ultimate.
    pub fn run_crt(&self, data: &[u8]) -> Result<()> {
        debug!("Run CRT file of {} bytes", data.len());
        self.post("runners:run_crt", data.to_vec())?;
        Ok(())
    }

    /// Emulate pressing the menu button
    pub fn menu(&self) -> Result<()> {
        debug!("Emulating menu button press");
        self.put("machine:menu_button")?;
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
        check_address_overflow(address, data.len() as u16)?;
        if matches!(address, 0 | 1) {
            warn!("DMA cannot access internal CPU registers at address 0 and 1");
        }
        let path = format!("machine:writemem?address={address:x}");
        self.post(&path, data.to_vec())?;
        debug!("Wrote {} byte(s) to {:#06x}", data.len(), address);
        Ok(())
    }

    /// Emulate keyboard input
    ///
    /// Done by injecting PETSCII bytes to the C64 input buffer.
    pub fn type_text(&self, s: &str) -> Result<()> {
        debug!("Emulating keyboard typing: {s}");
        // From the C64 Programmers Reference Guide, page 315-316:
        const KEYBOARD_LSTX: u16 = 0xc5; // Current key pressed (64 = no key pressed)
        const KEYBOARD_NDX: u16 = 0xc6; // Number of characters in keyboard buffer
        const KEYBOARD_BUFFER: u16 = 0x277; // Keyboard buffer queue (10 bytes)

        ensure!(
            self.basic_ready()?,
            "cannot emulate typing as BASIC prompt is not ready"
        );

        // Convert string to PETSCII bytes
        let petscii: Vec<u8> = s
            .chars()
            .map(|c| Petscii::from_str_lossy(&c.to_string())[0])
            .collect();

        // C64 input buffer is limited to 10 characters
        for chunk in petscii.chunks(10) {
            self.write_mem(KEYBOARD_LSTX, &[0, 0])?; // clear keyboard buffer
            self.write_mem(KEYBOARD_BUFFER, chunk)?; // write PETSCII to buffer
            self.write_mem(KEYBOARD_NDX, &[chunk.len() as u8])?; // trigger typing
            sleep(Duration::from_millis(20)); // wait for C64 (may not be needed)
        }
        Ok(())
    }

    /// Read word (2 bytes) from memory and interpret as little endian
    pub fn read_le_word(&self, address: u16) -> Result<u16> {
        let bytes: [u8; 2] = self
            .read_mem(address, 2)?
            .try_into()
            .map_err(|_| anyhow!("failed to read from {address:#06x}"))?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Check if BASIC prompt is active and accepts input
    ///
    /// Done by checking if the system vector at 0x0302 points the BASIN kernal routine.
    #[allow(unused)]
    fn basic_ready(&self) -> Result<bool> {
        return Ok(true);
        todo!("implement correct basic_ready check");
        const BASIN_ADDR: u16 = 0xa7ae; // BASIC input routine in Kernal ROM
        const VECTOR_ADDR: u16 = 0x0302; // System vector
        let word = self.read_le_word(VECTOR_ADDR)?;
        debug!("Word at {VECTOR_ADDR:#06x} is {word:#06x}");
        ensure!(
            word != 0,
            "BASIC prompt is not ready, vector at {VECTOR_ADDR:#06x} is zero"
        );
        Ok(self.read_le_word(VECTOR_ADDR)? == BASIN_ADDR)
    }

    /// Read `length` bytes from `address`
    pub fn read_mem(&self, address: u16, length: u16) -> Result<Vec<u8>> {
        check_address_overflow(address, length)?;
        if matches!(address, 0 | 1) {
            warn!("Warning: DMA cannot access internal CPU registers at address 0 and 1");
        }
        let path = format!("machine:readmem?address={address:x}&length={length}");
        let bytes = self.get(path.as_str())?.bytes()?.to_vec();
        debug!("Read {length} byte(s) from {address:#06x}");
        Ok(bytes)
    }

    /// Play SID file - if no `songnr` is provided, the default song is played.
    pub fn sid_play(&self, siddata: &[u8], songnr: Option<u8>) -> Result<()> {
        let path = match songnr {
            Some(songnr) => format!("runners:sidplay?songnr={songnr}"),
            None => "runners:sidplay".to_string(),
        };
        self.post(&path, siddata.to_vec())?;
        Ok(())
    }

    /// Play amiga MOD file
    pub fn mod_play(&self, moddata: &[u8]) -> Result<()> {
        self.post("runners:modplay", moddata.to_vec())?;
        Ok(())
    }

    /// Load data into memory using either a custom address, or deduce the
    /// load address from the first two bytes of the data (little endian).
    /// In the case of the latter, the first two bytes are not written to memory.
    /// Returns the load address and the number of bytes written.
    pub fn load_data(&self, data: &[u8], address: Option<u16>) -> Result<(u16, usize)> {
        match address {
            Some(address) => {
                self.write_mem(address, data)?;
                Ok((address, data.len()))
            }
            None => {
                let load_address = auxiliary::extract_load_address(data)?;
                self.write_mem(load_address, &data[2..])?; // skip first two bytes
                Ok((load_address, data.len() - 2))
            }
        }
    }

    /// Get drive list
    pub fn drive_list(&self) -> Result<HashMap<String, Drive>> {
        let response = self.get("drives")?;
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
        let url = format!("{}/drives/{drive}:mount", self.url_prefix);
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", path)
            .map_err(|e| anyhow!("disk image error: {e}"))?
            .text("mode", mount_mode.to_string())
            .text("type", disktype.to_string());

        let response = self
            .client
            .post(url)
            .multipart(form)
            .headers(self.headers.clone())
            .send()?;

        Self::check_response(&response)?;

        // should not trigger by normal operation and indicates a problem
        // with the request or the server
        if response.status().is_client_error() {
            bail!(
                "disk mount error: {} - {}",
                response.status(),
                response.text().unwrap()
            );
        }
        // optionally reset and run the first program on the disk
        // a short delay is needed to allow the reset to complete
        if run {
            self.reset()?;
            sleep(Duration::from_secs(3));
            self.type_text("load\"*\",8,1\nrun\n")?;
        }
        Ok(())
    }

    /// Start video, audio, or debug streaming
    pub fn start_stream(&self, host: &Host, port: u16, kind: StreamType) -> Result<()> {
        self.put(&format!("streams/{kind}:start?ip={host}:{port}"))?;
        Ok(())
    }

    /// Start video, audio, or debug streaming
    pub fn stop_stream(&self, kind: StreamType) -> Result<()> {
        self.put(&format!("streams/{kind}:stop"))?;
        Ok(())
    }
}
