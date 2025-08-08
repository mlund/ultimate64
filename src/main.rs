use anyhow::{bail, ensure, Result};
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use log::debug;
use parse_int::parse;
use ultimate64::aux;
use ultimate64::drives::Drive;
use ultimate64::{drives, Rest};
extern crate pretty_env_logger;
use pretty_env_logger::env_logger::DEFAULT_FILTER_ENV;
use prettytable::{format, Cell, Row, Table};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use url::Host;

// Clap 4 colors: https://github.com/clap-rs/clap/issues/3234#issuecomment-1783820412
fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

/// Helper function to determine if file has a disk image extension
fn has_disk_image_extension<P: AsRef<Path>>(file: P) -> Result<()> {
    drives::DiskImageType::from_file_name(file).map(|_| ())
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ultimate64")]
#[command(author = "Mikael Lund aka Wombat")]
#[command(about = "Network Control for Ultimate series", version)]
#[command(color = clap::ColorChoice::Auto)]
#[command(styles=styles())]
struct Cli {
    /// IP address or hostname of ultimate device
    #[clap(env = "ULTIMATE_HOST")]
    #[arg(value_parser = Host::parse)]
    host: Host,
    /// Subcommand to run
    #[command(subcommand)]
    command: Commands,
    /// Verbose output
    #[clap(long, short = 'v', action)]
    pub verbose: bool,
    /// Optional password for Ultimate device
    #[clap(env = "ULTIMATE_PASSWORD")]
    #[clap(long, short = 'p')]
    pub password: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show drive information
    Drives,
    /// Show Ultimate device information
    Info,
    /// Load file into memory
    Load {
        /// File to load
        file: PathBuf,
        /// Load address; otherwise deduce from first two bytes in file
        #[clap(long, short = '@', default_value = None)]
        #[arg(value_parser = parse::<u16>)]
        address: Option<u16>,
        /// Attempt to run after loading using RUN or SYS
        #[clap(long, short = 'r', action, default_value_t = false)]
        run: bool,
        /// Reset before loading
        #[clap(long, action, default_value_t = false)]
        reset: bool,
    },
    /// Press menu button
    Menu,
    /// Mount disk image
    Mount {
        /// Image file
        file: PathBuf,
        /// Drive number
        #[clap(long, short = 'd', default_value = "a")]
        drive: String,
        /// Mount mode
        #[clap(long, short = 'm', default_value = "ro")]
        #[arg(value_enum)]
        mode: drives::MountMode,
        /// Reset and run after mounting
        #[clap(long, short = 'r', action, default_value_t = false)]
        run: bool,
    },

    /// Pause machine
    Pause,
    /// Read memory
    Peek {
        /// Address to read from, e.g. `4096` or `0x1000`
        #[arg(value_parser = parse::<u16>)]
        address: u16,
        /// Number of bytes to read
        #[clap(long, short = 'n', default_value = "1")]
        #[arg(value_parser = parse::<u16>)]
        length: u16,
        /// Write to binary file instead of hexdump
        #[clap(long, short = 'o')]
        outfile: Option<PathBuf>,
        /// Disassemble instead of hexdump
        #[clap(long = "dasm", short = 'd', action, conflicts_with = "outfile")]
        disassemble: bool,
    },
    /// Play SID or Amiga MOD file
    Play {
        /// SID or MOD file
        file: PathBuf,
        /// Optional song number for SID
        #[clap(short = 'n')]
        #[arg(value_parser = parse::<u8>)]
        songnr: Option<u8>,
    },
    /// Write or modify byte(s) in memory
    Poke {
        /// Address to write to, e.g. `4096` or `0x1000`
        #[arg(value_parser = parse::<u16>)]
        address: u16,
        /// Value to write, e.g. `16`, `0x10` or `0b0001_0000`
        #[arg(value_parser = parse::<u8>)]
        value: u8,
        /// Bitwise AND with existing value
        #[clap(long = "and", action, conflicts_with_all = ["bitwise_or", "bitwise_xor"])]
        bitwise_and: bool,
        /// Bitwise OR with existing value
        #[clap(long = "or", action, conflicts_with_all = ["bitwise_and", "bitwise_xor"])]
        bitwise_or: bool,
        /// Bitwise XOR with existing value
        #[clap(long = "xor", action, conflicts_with_all = ["bitwise_and", "bitwise_or"])]
        bitwise_xor: bool,
        #[clap(long, short = 'f', conflicts_with_all = ["bitwise_and", "bitwise_or", "bitwise_xor"])]
        #[arg(value_parser = parse::<u16>)]
        /// Fill n bytes with value
        fill: Option<u16>,
    },
    /// Power off machine
    Poweroff,
    /// Reboot machine
    Reboot,
    /// Reset machine
    Reset,
    /// Resume machine
    Resume,
    /// Load and run PRG or CRT file
    #[command(arg_required_else_help = true)]
    Run {
        /// PRG or CRT file to load and run
        file: PathBuf,
    },
    /// Emulate keyboard input
    Type {
        /// Unicode text to type - will be converted to PETSCII
        text: String,
    },
}

/// Disassemble `length` bytes from memory, starting at `address`
/// # Panics
/// Panics if the disassembler fails to disassemble the bytes
fn print_disassembled(bytes: &[u8], address: u16) -> Result<()> {
    disasm6502::from_addr_array(bytes, address)
        .unwrap()
        .iter()
        .for_each(|line| {
            println!("{line}");
        });
    Ok(())
}

fn do_main() -> Result<()> {
    let args = Cli::parse();
    let ultimate = Rest::new(&args.host, args.password.clone())?;

    if args.verbose && std::env::var(DEFAULT_FILTER_ENV).is_err() {
        std::env::set_var(DEFAULT_FILTER_ENV, "Debug");
    } else {
        std::env::set_var(DEFAULT_FILTER_ENV, "Info");
    }
    pretty_env_logger::init();

    match args.command {
        Commands::Drives => {
            let drives = ultimate.drive_list()?;
            print_drive_table(drives);
        }
        Commands::Info => {
            let info = ultimate.info()?;
            println!("{info}");
        }
        Commands::Pause => {
            ultimate.pause()?;
        }
        Commands::Poweroff => {
            ultimate.poweroff()?;
        }
        Commands::Peek {
            address,
            length,
            outfile,
            disassemble,
        } => {
            let data = ultimate.read_mem(address, length)?;
            if disassemble {
                print_disassembled(&data, address)?;
            } else if outfile.is_some() {
                fs::write(outfile.unwrap(), &data)?;
            } else {
                data.iter().for_each(|byte| {
                    print!("{byte:#04x} ");
                });
                println!()
            }
        }
        Commands::Poke {
            address,
            value,
            bitwise_and,
            bitwise_or,
            bitwise_xor,
            fill,
        } => {
            if let Some(fill) = fill {
                ensure!(fill > 0, "fill must be greater than zero");
                let data = vec![value; fill as usize];
                ultimate.write_mem(address, &data)?;
                debug!(
                    "Filled [{:#06x}-{:#06x}] with {:#04x}",
                    address,
                    address + fill - 1,
                    value
                );
                return Ok(());
            };
            let value = if bitwise_and {
                ultimate.read_mem(address, 1)?[0] & value
            } else if bitwise_or {
                ultimate.read_mem(address, 1)?[0] | value
            } else if bitwise_xor {
                ultimate.read_mem(address, 1)?[0] ^ value
            } else {
                value
            };
            debug!("Poke {value:#04x} to {address:#06x}");
            ultimate.write_mem(address, &[value])?;
        }
        Commands::Reboot => {
            ultimate.reboot()?;
        }
        Commands::Reset => {
            ultimate.reset()?;
        }
        Commands::Resume => {
            ultimate.resume()?;
        }
        Commands::Run { file } => {
            let data = fs::read(&file)?;
            match aux::get_extension(&file).unwrap_or_default().as_str() {
                "crt" => ultimate.run_crt(&data)?,
                _ => ultimate.run_prg(&data)?,
            }
        }
        Commands::Play { file, songnr } => {
            let data = fs::read(&file)?;
            let ext = aux::get_extension(&file).unwrap_or_default();
            match ext.as_str() {
                "sid" => ultimate.sid_play(&data, songnr)?,
                "mod" => ultimate.mod_play(&data)?,
                _ => bail!("Unsupported music file format: {ext}"),
            }
        }
        Commands::Type { text } => {
            ultimate.type_text(&text)?;
        }
        Commands::Menu => {
            ultimate.menu()?;
        }
        Commands::Mount {
            file,
            drive: drive_id,
            mode,
            run,
        } => {
            has_disk_image_extension(&file)?;
            ultimate.mount_disk_image(&file, drive_id, mode, run)?;
        }
        Commands::Load {
            file,
            address,
            run,
            reset,
        } => {
            let data = fs::read(file)?;

            if reset {
                ultimate.reset()?;
            }

            let (address, _) = ultimate.load_data(&data, address)?;

            if run {
                const BASIC_LOAD_ADDR: u16 = 0x0801;
                if address == BASIC_LOAD_ADDR {
                    ultimate.type_text("run\n")?;
                } else {
                    ultimate.type_text(&format!("sys{address}\n"))?
                }
            }
        }
    }
    Ok(())
}

fn print_drive_table(drives: HashMap<String, Drive>) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Drive"),
        Cell::new("Id"),
        Cell::new("Type"),
        Cell::new("Enabled"),
        Cell::new("Image file"),
    ]));

    for (name, drive) in drives {
        table.add_row(Row::new(vec![
            Cell::new(&name),
            Cell::new(&drive.bus_id.to_string()),
            Cell::new(
                &drive
                    .drive_type
                    .map_or("Unknown".to_string(), |t| t.to_string()),
            ),
            Cell::new(&drive.enabled.to_string()),
            Cell::new(drive.image_file.as_deref().unwrap_or("")),
        ]));
    }

    table.printstd();
}

fn main() {
    if let Err(err) = do_main() {
        eprintln!("Error: {}", &err);
        std::process::exit(1);
    }
}
