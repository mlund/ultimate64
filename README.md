[![Crates.io](https://img.shields.io/crates/v/ultimate64)](https://crates.io/crates/ultimate64)
[![Rust](https://github.com/mlund/ultimate64/actions/workflows/rust.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/mlund/ultimate64/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/rust-clippy.yml)
[![.github/workflows/release.yml](https://github.com/mlund/ultimate64/actions/workflows/release.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/release.yml)
[![docs.rs](https://img.shields.io/docsrs/ultimate64)](https://docs.rs/ultimate64/latest/ultimate64)

# Ultimate64

Rust library and command line interface for communicating with [Ultimate-64 and Ultimate-II+](https://ultimate64.com) devices using
the [REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html).

# Installation

Either download a [pre-compiled binary](https://github.com/mlund/ultimate64/releases/latest),
or compile and install using `cargo`, provided that you have a working
[Rust](https://www.rust-lang.org/tools/install) installation:

~~~ bash
cargo install ultimate64
~~~

# Usage

~~~ bash
ultimate64 HOST COMMAND <OPTIONS>
~~~

Where `HOST` is the IP address or hostname of the Ultimate device on your local network.
Alternatively specify this in the environmental variable `ULTIMATE_HOST` as
assumed in the following examples.

## Examples

~~~ bash
ultimate64 --help                            # show available commands
ultimate64 pause                             # pause machine
ultimate64 run skate_or_die.prg              # load and run PRG file
ultimate64 load sprites.dat --address 0x2000 # load data to memory
ultimate65 peek 0x1000 --dasm -n 32          # disassemble memory
ultimate65 poke 0xd020 3                     # write single byte
ultimate65 poke 4096 --xor 0b0000_1100       # bitwise manipulation
ultimate65 poke 0x0400 0x20 --fill 1000      # fill memory
ultimate64 sidplay yie_ar_kung_fu.sid -n 2   # play SID tune
ultimate64 modplay enigma.mod                # play Amiga MOD tune
~~~

Addresses can be hexadecimal (`0x1000`) or decimal (`4096`).

# Features

- [x] Compiled, small, and cross platform with no external dependencies
- [x] Can be used either as a CLI tool or as a library
- [x] Modern CLI with subcommands
- [x] First class memory safety due to Rust
- [x] Excellent error handling; error messages; and input validation
- [x] Convenient decimal, hexadecimal, and binary input
- [x] Bitwise operations for memory manipulation
- [x] 6502 disassembly
- [x] Load address detection
- [x] Precompiled binaries for MacOS, Linux, and Windows

## Todo

- [ ] Disk image and file manipulation
- [ ] Memory bank switching for RAM access
- [ ] Ultimate configuration handling
