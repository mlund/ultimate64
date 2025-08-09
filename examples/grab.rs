// udp_multicast_grab.rs
// Rust port of the provided Python script.
//
// Cargo.toml dependencies (add to your Cargo.toml):
// [dependencies]
// image = "0.24"
// socket2 = "0.4"
// byteorder = "1.4"
//
// Build with: cargo build --release
// Run with: cargo run --release -- <numFrames>

use byteorder::{ByteOrder, LittleEndian};
use image::{ImageBuffer, Rgb};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::ops::BitAnd;
use std::time::Duration;
// use anyhow::{Result, Ok};

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
    let multicast_group = Ipv4Addr::new(239, 0, 1, 64);
    let listen_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 11000);

    // Create a UDP socket using socket2 so we can join multicast groups
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    // On some platforms you may also need set_reuse_port(true).
    #[cfg(target_family = "unix")]
    {
        socket.set_reuse_port(true).ok();
    }

    socket.bind(&listen_addr.into())?;

    // join multicast group on all interfaces
    socket.join_multicast_v4(&multicast_group, &Ipv4Addr::UNSPECIFIED)?;

    // convert to std::net::UdpSocket for recv_from etc.
    let udp_socket: UdpSocket = socket.into(); //.into_udp_socket();
    udp_socket.set_read_timeout(Some(Duration::from_millis(200)))?;

    let mut buf = [0u8; 1024];

    // initial receive loop (mirrors the Python script behavior)
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((len, _addr)) => {
                if len >= 8 {
                    // struct.unpack("<HHHH", data[0:8]) -> seq, frame, lin, width
                    if LittleEndian::read_u16(&buf[4..6]).bitand(0x8000) != 0 {
                        break;
                    }
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                // timeout; continue trying
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    let mut frame: Vec<u8> = Vec::new();

    loop {
        let (len, _addr) = match udp_socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                continue
            }
            Err(e) => return Err(e),
        };

        const HEADER_LEN: usize = 12;
        if len >= HEADER_LEN {
            frame.extend_from_slice(&buf[HEADER_LEN..len]);
            if LittleEndian::read_u16(&buf[4..6]).bitand(0x8000) != 0 {
                break;
            }
        }
    }

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

            let c_lo = COLORS[lo]; //.get(lo).copied().unwrap_or([0, 0, 0]);
            let c_hi = COLORS[hi]; //.get(hi).copied().unwrap_or([0, 0, 0]);

            img.put_pixel((2 * x) as u32, y as u32, Rgb(c_lo));
            img.put_pixel((2 * x + 1) as u32, y as u32, Rgb(c_hi));
            i += 1;
        }

        img.save("grab.png")
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
    }

    Ok(())
}
