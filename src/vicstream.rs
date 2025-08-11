//! # VIC stream capturing

use anyhow::{anyhow, bail, Ok, Result};
use byteorder::{ByteOrder, LittleEndian};
use image::DynamicImage;
use image::{imageops::FilterType, ImageBuffer, Rgb};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::ops::BitAnd;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

/// End of frame marker (bit 15 set)
const END_OF_FRAME: u16 = 1 << 15;
/// Position of line number (two bytes, little-endian) in the VIC frame header
const LINE_NUM_POS: usize = 4;
/// Header length
const HEADER_LEN: usize = 12;

static COLORS: &[Rgb<u8>] = &[
    Rgb([0x00, 0x00, 0x00]),
    Rgb([0xEF, 0xEF, 0xEF]),
    Rgb([0x8D, 0x2F, 0x34]),
    Rgb([0x6A, 0xD4, 0xCD]),
    Rgb([0x98, 0x35, 0xA4]),
    Rgb([0x4C, 0xB4, 0x42]),
    Rgb([0x2C, 0x29, 0xB1]),
    Rgb([0xEF, 0xEF, 0x5D]),
    Rgb([0x98, 0x4E, 0x20]),
    Rgb([0x5B, 0x38, 0x00]),
    Rgb([0xD1, 0x67, 0x6D]),
    Rgb([0x4A, 0x4A, 0x4A]),
    Rgb([0x7B, 0x7B, 0x7B]),
    Rgb([0x9F, 0xEF, 0x93]),
    Rgb([0x6D, 0x6A, 0xEF]),
    Rgb([0xB2, 0xB2, 0xB2]),
];

/// Takes a single snap-shot of the C64 screen
///
/// If no file path is given, the snapshot will be printed to the console.
pub fn take_snapshot(url: &Url, image_file: Option<&Path>, scale: Option<u32>) -> Result<()> {
    let udp_socket = get_socket(url)?;
    let frame = capture_frame(udp_socket)?;
    let img = make_image(&frame);
    let img = scale_image(img, scale)?;

    if let Some(path) = image_file {
        img.save(path)
            .map_err(|e| anyhow!("Failed to save image: {}", e))?;
    } else {
        // Print image to console on supported terminal
        let conf = viuer::Config {
            width: Some(60),
            ..Default::default()
        };
        let img = DynamicImage::ImageRgb8(img);
        viuer::print(&img, &conf)?;
    }

    Ok(())
}

pub fn get_socket(url: &Url) -> Result<UdpSocket> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("Invalid URL host: {}", url))?;
    let port = url.port().unwrap_or(11000);
    let multicast_group = Ipv4Addr::from_str(host)?;
    let listen_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(target_family = "unix")]
    {
        socket.set_reuse_port(true).ok();
    }
    socket.bind(&listen_addr.into())?;
    socket.join_multicast_v4(&multicast_group, &Ipv4Addr::UNSPECIFIED)?;

    let udp_socket: UdpSocket = socket.into();
    udp_socket.set_read_timeout(Some(Duration::from_millis(200)))?;
    Ok(udp_socket)
}

/// Capture single VIC frame
pub fn capture_frame(udp_socket: UdpSocket) -> Result<Vec<u8>> {
    use std::result::Result::Ok;
    let mut frame: Vec<u8> = Vec::with_capacity(384 * 272 / 2);
    let mut buf = [0; 1024];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok(_) => {
                if bit15_is_set(&buf) {
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::TimedOut => continue,
            Err(e) => bail!("Failed to receive VIC data: {}", e),
        }
    }
    loop {
        let (len, _addr) = match udp_socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e) if e.kind() == ErrorKind::TimedOut => continue,
            Err(e) => bail!("Failed to receive VIC data: {}", e),
        };

        if len >= HEADER_LEN {
            frame.extend_from_slice(&buf[HEADER_LEN..len]);
            if bit15_is_set(&buf) {
                break;
            }
        }
    }
    Ok(frame)
}

fn bit15_is_set(buf: &[u8]) -> bool {
    buf.len() >= HEADER_LEN
        && LittleEndian::read_u16(&buf[LINE_NUM_POS..LINE_NUM_POS + 2]).bitand(END_OF_FRAME) != 0
}

pub fn make_image(frame: &[u8]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    const IMAGE_WIDTH: usize = 384;
    const BYTES_PER_ROW: usize = IMAGE_WIDTH / 2;
    let rows = frame.len() / BYTES_PER_ROW;
    let mut img = ImageBuffer::new(IMAGE_WIDTH as u32, rows as u32);
    let mut i: usize = 0;

    for y in 0..rows {
        for x in 0..BYTES_PER_ROW {
            if i >= frame.len() {
                break;
            }
            let byte = frame[i];
            let (lo, hi) = ((byte & 0xf) as usize, (byte >> 4) as usize);
            img.put_pixel((2 * x) as u32, y as u32, COLORS[lo]);
            img.put_pixel((2 * x + 1) as u32, y as u32, COLORS[hi]);

            i += 1;
        }
    }
    img
}

/// Scale image with optional scaling factor
fn scale_image(
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    scale: Option<u32>,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    match scale {
        Some(1) | None => Ok(img),
        Some(s) => Ok(image::imageops::resize(
            &img,
            img.width() * s,
            img.height() * s,
            FilterType::Nearest,
        )),
    }
}
