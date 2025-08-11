[![Crates.io](https://img.shields.io/crates/v/ultimate64)](https://crates.io/crates/ultimate64)
[![Rust](https://github.com/mlund/ultimate64/actions/workflows/rust.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/mlund/ultimate64/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/rust-clippy.yml)
[![.github/workflows/release.yml](https://github.com/mlund/ultimate64/actions/workflows/release.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/release.yml)
[![docs.rs](https://img.shields.io/docsrs/ultimate64)](https://docs.rs/ultimate64/latest/ultimate64)

# Ultimate64

Rust library and command line interface for communicating with
[Ultimate-64 and Ultimate-II+](https://ultimate64.com) devices using the
[REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html).

# Installation

Works with Linux, macOS, Windows.
Either download a [pre-compiled binary](https://github.com/mlund/ultimate64/releases/latest),
or compile and install using `cargo`, provided that you have a working
[Rust](https://www.rust-lang.org/tools/install) installation:

~~~ bash
cargo install ultimate64
~~~

# Usage

~~~ bash
ru64 HOST COMMAND <OPTIONS>
~~~

Where `HOST` is the IP address or hostname of the Ultimate device on your local network.
Alternatively specify this in the environmental variable `ULTIMATE_HOST` as
assumed in the following examples.
`ULTIMATE_PASSWORD` may be used to specify a network password for your Ultimate device.

## Examples

~~~ bash
ru64 --help                            # show available commands
ru64 info                              # Show device info (type, core version etc.)
ru64 run skate_or_die.prg              # load and run external PRG file
ru64 mount desert_dream.d64 --run      # mount external image and run
ru64 play yie_ar_kung_fu.sid -n 2      # play SID tune
ru64 play enigma.mod                   # play Amiga MOD tune
ru64 load sprites.dat --address 0x2000 # load data to memory
ru64 peek 0xa7ae --dasm -n 32          # disassemble memory
ru64 poke 0xd020 3                     # write single byte
ru64 poke 4096 --xor 0b0000_1100       # bitwise manipulation
ru64 poke 0x0400 0x20 --fill 1000      # fill memory
ru64 type $'print "hello"\n'           # Emulate keyboard typing
ru64 pause                             # pause machine
ru64 reset                             # reset machine
ru64 stream -n video --start           # start VIC video stream
ru64 screenshot -o screen.png          # take image snapshot of VIC stream
~~~

Addresses can be hexadecimal (`0x1000`) or decimal (`4096`).

### Experimental GUI

An experimental, cross-platform GUI is available with `cargo run --release --example egui`. Requires a Rust installation, see above. Currently only a VIC stream viewer is implemented.


# Features

- [x] Compiled, small, and cross platform with no external dependencies
- [x] Mount and run external disk images
- [x] Play SID and MOD files
- [x] Remote screenshots to disk or, if support by terminal, the console
- [x] Emulate keyboard typing w. unicode to PETSCII conversion
- [x] Convenient decimal, hexadecimal, and binary input
- [x] Bitwise operations for memory manipulation
- [x] 6502 disassembly
- [x] Load address detection
- [x] Network password support
- [x] First class memory safety due to Rust
- [x] Modern CLI with subcommands
- [x] Excellent error handling; error messages; and input validation
- [x] Can be used either as a CLI tool or as a library
- [x] Precompiled binaries for Linux and Windows (mac users should use `cargo`, see above)

## Todo

- [ ] Disk image manipulation
- [ ] Memory bank switching for RAM access
- [ ] Ultimate configuration handling
