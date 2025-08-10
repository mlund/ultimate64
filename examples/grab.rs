use byteorder::{ByteOrder, LittleEndian};
use image::{imageops::FilterType, ImageBuffer, Rgb};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::ops::BitAnd;
use std::time::Duration;
// use anyhow::{Result, Ok};

const END_OF_FRAME: u16 = 1 << 15;
const LINE_NUMBER_POS: usize = 4;
const HEADER_LEN: usize = 12;

static COLORS: &[[u8; 3]] = &[
    [0x00, 0x00, 0x00],
    [0xEF, 0xEF, 0xEF],
    [0x8D, 0x2F, 0x34],
    [0x6A, 0xD4, 0xCD],
    [0x98, 0x35, 0xA4],
    [0x4C, 0xB4, 0x42],
    [0x2C, 0x29, 0xB1],
    [0xEF, 0xEF, 0x5D],
    [0x98, 0x4E, 0x20],
    [0x5B, 0x38, 0x00],
    [0xD1, 0x67, 0x6D],
    [0x4A, 0x4A, 0x4A],
    [0x7B, 0x7B, 0x7B],
    [0x9F, 0xEF, 0x93],
    [0x6D, 0x6A, 0xEF],
    [0xB2, 0xB2, 0xB2],
];

static _COLORS2: &[[u8; 3]] = &[
    [0xF0, 0xF0, 0xF0],
    [0x00, 0x00, 0x00],
    [0x8D, 0x2F, 0x34],
    [0x6A, 0xD4, 0xCD],
    [0x98, 0x35, 0xA4],
    [0x4C, 0xB4, 0x42],
    [0x2C, 0x29, 0xB1],
    [0xA0, 0x90, 0x00],
    [0x98, 0x4E, 0x20],
    [0x5B, 0x38, 0x00],
    [0xD1, 0x67, 0x6D],
    [0x99, 0x99, 0x99],
    [0x66, 0x66, 0x66],
    [0x20, 0xA0, 0x20],
    [0x20, 0x20, 0xA0],
    [0x33, 0x33, 0x33],
];

fn main() -> io::Result<()> {
    let udp_socket = get_socket()?;

    let frame = match capture_frame(udp_socket) {
        Ok(value) => value,
        Err(value) => return value,
    };

    save_png(&frame, None)?;

    Ok(())
}

fn get_socket() -> Result<UdpSocket, io::Error> {
    let multicast_group = Ipv4Addr::new(239, 0, 1, 64);
    let listen_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 11000);
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

fn capture_frame(udp_socket: UdpSocket) -> Result<Vec<u8>, Result<(), io::Error>> {
    let mut frame: Vec<u8> = Vec::with_capacity(384 * 272 / 2);
    let mut buf = [0u8; 1024];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok(_) => {
                if bit15_is_set(&buf) {
                    break;
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => continue,
            Err(e) => return Err(Err(e)),
        }
    }
    loop {
        let (len, _addr) = match udp_socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(ref e) if e.kind() == ErrorKind::TimedOut => continue,
            Err(e) => return Err(Err(e)),
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
        && LittleEndian::read_u16(&buf[LINE_NUMBER_POS..LINE_NUMBER_POS + 2]).bitand(END_OF_FRAME)
            != 0
}

fn save_png(frame: &[u8], scale_factor: Option<u32>) -> Result<(), io::Error> {
    const IMAGE_WIDTH: usize = 384;
    const BYTES_PER_ROW: usize = IMAGE_WIDTH / 2;
    let rows = frame.len() / BYTES_PER_ROW;
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(IMAGE_WIDTH as u32, rows as u32);
    let mut i: usize = 0;
    for y in 0..rows {
        for x in 0..BYTES_PER_ROW {
            if i >= frame.len() {
                break;
            }
            let b = frame[i] as usize;
            let (lo, hi) = ((b & 0xF) as usize, (b >> 4) as usize);
            let c_lo = COLORS[lo];
            let c_hi = COLORS[hi];

            img.put_pixel((2 * x) as u32, y as u32, Rgb(c_lo));
            img.put_pixel((2 * x + 1) as u32, y as u32, Rgb(c_hi));
            i += 1;
        }
    }
    let scale = scale_factor.unwrap_or(1);
    let img = image::imageops::resize(
        &img,
        img.width() * scale,
        img.height() * scale,
        FilterType::Nearest, // keeps pixel edges crisp
    );
    img.save("grab.png")
        .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
    Ok(())
}
