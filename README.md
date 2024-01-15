[![Rust](https://github.com/mlund/ultimate64/actions/workflows/rust.yml/badge.svg)](https://github.com/mlund/ultimate64/actions/workflows/rust.yml)

# Ultimate64

Rust library and command line interface for interfacing with [Ultimate-64 and Ultimate-II](https://ultimate64.com) devices using
the [REST API](https://1541u-documentation.readthedocs.io/en/latest/api/api_calls.html).

# Installation

Currently no binaries are provided and you need to first install [Rust](https://www.rust-lang.org/tools/install).
Then compile and install with:

~~~ bash
cargo install ultimate64
~~~

# Usage

~~~ bash
ultimate64 HOST COMMAND <OPTIONS>
~~~

Where `HOST` is the IP address or hostname of the Ultimate device on your local network.
Alternatively specify this in the envronmental variable `ULTIMATE_HOST` as
assumed in the following examples.

## Examples

~~~ bash
ultimate64 --help                            # show available commands
ultimate64 pause                             # pause machine
ultimate64 prg skate_or_die.prg              # load and run PRG file
ultimate64 load sprites.dat --address 0x2000 # load data to memory
ultimate65 peek 0x1000 --dasm -n 32          # disassemble 32 bytes
ultimate65 poke 0xd020 3                     # write single byte
ultimate64 sidplay yie_ar_kung_fu.sid -n 2   # play SID tune
ultimate64 modplay enigma.mod                # play Amiga MOD tune
~~~

Addresses can be decimal (`4096`) or hexadecimal (`0x1000`).

# Features

- [x] Compiled, small, and cross platform with no external dependencies
- [x] Can be used either as a CLI tool or as a library
- [x] Modern CLI with subcommands
- [x] World class memory safety due to Rust
- [x] Excellent error handling and error messages
- [x] Convenient decimal, hexademical, binary and even octal input
- [x] 6502 disassembly
- [x] Load address detection

## Todo

- [ ] Binary distribution for MacOS, Linux, and Windows
- [ ] Disk image and file manipulation
