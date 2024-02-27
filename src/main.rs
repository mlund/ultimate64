use anyhow::Result;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use log::debug;
use parse_int::parse;
use ultimate64::aux;
use ultimate64::{drives, Rest};
extern crate pretty_env_logger;
use pretty_env_logger::env_logger::DEFAULT_FILTER_ENV;
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
    /// Network address of ultimate device
    #[clap(env = "ULTIMATE_HOST")]
    #[arg(value_parser = Host::parse)]
    host: Host,
    /// Subcommand to run
    #[command(subcommand)]
    command: Commands,
    /// Verbose output
    #[clap(long, short = 'v', action)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
enum DiskImageCmd {
    /// Mount disk image (unfinished)
    Mount {
        /// Image file
        file: PathBuf,
        /// Drive number
        #[clap(long, short = 'i', default_value = "8")]
        #[arg(value_parser = parse::<u8>)]
        drive_id: u8,
        /// Mount mode
        #[clap(long, short = 'm', default_value = "ro")]
        #[arg(value_enum)]
        mode: drives::MountMode,
    },
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show drive information
    Drives,
    /// Disk image operations (experimental)
    Image {
        #[clap(subcommand)]
        command: DiskImageCmd,
    },
    /// Load file into memory
    Load {
        /// File to load
        file: PathBuf,
        /// Load address; otherwise deduce from first two bytes in file
        #[clap(long, short = '@', default_value = None)]
        #[arg(value_parser = parse::<u16>)]
        address: Option<u16>,
    },
    /// Play Amiga MOD file
    Modplay {
        /// MOD file
        file: PathBuf,
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
    /// Write single byte to memory
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
    /// Play SID file
    Sidplay {
        /// SID file
        file: PathBuf,
        /// Optional song number
        #[clap(short = 'n')]
        #[arg(value_parser = parse::<u8>)]
        songnr: Option<u8>,
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
            println!("{}", line);
        });
    Ok(())
}

fn do_main() -> Result<()> {
    let args = Cli::parse();
    let ultimate = Rest::new(&args.host);

    if args.verbose && std::env::var(DEFAULT_FILTER_ENV).is_err() {
        std::env::set_var(DEFAULT_FILTER_ENV, "Debug");
    } else {
        std::env::set_var(DEFAULT_FILTER_ENV, "Info");
    }
    pretty_env_logger::init();

    match args.command {
        Commands::Drives => {
            let drives = ultimate.drives()?;
            println!("{}", drives);
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
                std::fs::write(outfile.unwrap(), &data)?;
            } else {
                data.iter().for_each(|byte| {
                    print!("{:#04x} ", byte);
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
            if fill.is_some() && fill.unwrap() > 0 {
                let fill = fill.unwrap();
                let data = vec![value; fill as usize];
                ultimate.write_mem(address, &data)?;
                debug!(
                    "Filled [{:#06x}-{:#06x}] with {:#04x}",
                    address,
                    address + fill - 1,
                    value
                );
                return Ok(());
            }
            let value = if bitwise_and {
                ultimate.read_mem(address, 1)?[0] & value
            } else if bitwise_or {
                ultimate.read_mem(address, 1)?[0] | value
            } else if bitwise_xor {
                ultimate.read_mem(address, 1)?[0] ^ value
            } else {
                value
            };
            debug!("Poke {:#04x} to {:#06x}", value, address);
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
            let data = std::fs::read(&file)?;
            match aux::get_extension(&file).unwrap_or_default().as_str() {
                "crt" => ultimate.run_crt(&data)?,
                _ => ultimate.run_prg(&data)?,
            }
        }
        Commands::Sidplay { file, songnr } => {
            let data = std::fs::read(file)?;
            ultimate.sid_play(&data, songnr)?;
        }
        Commands::Modplay { file } => {
            let data = std::fs::read(file)?;
            ultimate.mod_play(&data)?;
        }
        Commands::Load { file, address } => {
            let data = std::fs::read(file)?;
            ultimate.load_data(&data, address)?;
        }
        Commands::Image { command } => match command {
            DiskImageCmd::Mount {
                file,
                drive_id,
                mode,
            } => {
                has_disk_image_extension(&file)?;
                ultimate.mount_disk_image(&file, drive_id, mode)?;
            }
        },
    }
    Ok(())
}

fn main() {
    if let Err(err) = do_main() {
        eprintln!("Error: {}", &err);
        std::process::exit(1);
    }
}
