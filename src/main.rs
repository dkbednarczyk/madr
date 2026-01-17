mod debounce;
mod device;
mod dpi;
mod info;
mod performance;
mod sensor;
mod sleep;

use anyhow::anyhow;
use anyhow::Result;
use std::thread;
use std::time::Duration;

use clap::{builder::PossibleValuesParser, value_parser, Parser, Subcommand};
use device::Device;

#[derive(Parser)]
#[command(name = "vxectl")]
#[command(about = "Control your VXE gaming mouse from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// test
    Test,

    /// Set device parameters
    Set {
        /// DPI stage to enable
        #[arg(short = 'd', long, value_parser = value_parser!(u8).range(1..=8))]
        dpi_stage: Option<u8>,

        /// Polling rate in Hz
        #[arg(short = 'p', long, value_parser = PossibleValuesParser::new(["125", "250", "500", "1000", "2000", "4000", "8000"]))]
        polling_rate: Option<String>,

        /// Sensor setting to enable
        #[arg(short = 'x', long, value_parser = PossibleValuesParser::new(["basic", "competitive", "max"]))]
        sensor_setting: Option<String>,

        /// Debounce time in milliseconds
        #[arg(short = 'b', long, value_parser = PossibleValuesParser::new(["0", "1", "2", "4", "8", "15", "20"]))]
        debounce: Option<String>,

        /// Sleep timeout (inactivity before sleep)
        #[arg(short = 's', long, value_parser = PossibleValuesParser::new(["30s", "1m", "2m", "3m", "5m", "20m", "25m", "30m"]))]
        sleep: Option<String>,
    },

    /// Change dpi settings
    #[clap(subcommand)]
    Dpi(Dpi),

    /// Get device info
    #[clap(subcommand)]
    Info(Info),
}

#[derive(Subcommand)]
enum Info {
    /// Get battery status
    Battery,
    /// Get sensor settings
    Sensor,
}

#[derive(Subcommand)]
enum Dpi {
    /// Set DPI for a specific stage
    Set {
        /// DPI stage to change (1-8)
        #[arg(short, long, value_parser = value_parser!(u8).range(1..=8))]
        stage: u8,
        /// X DPI value
        #[arg(short, long, value_parser = value_parser!(u16).range(50..=16000))]
        x_dpi: u16,
        /// Y DPI value, if not specified, X DPI will be used
        #[arg(short, long, value_parser = value_parser!(u16).range(50..=16000))]
        y_dpi: Option<u16>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let device = match Device::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Test => {
            // dpi setting 1
            device.write(&vec![
                0x8, 0x8, 0x0, 0x0, 0xc, 0x8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                0x31,
            ])?;
            let mut buf = [0u8; 64];
            let len = device.read(&mut buf)?;
            println!("Response: {:02x?}", &buf[..len]);
        }
        Commands::Set {
            dpi_stage,
            polling_rate,
            sensor_setting,
            debounce,
            sleep,
        } => {
            // Validate polling rate for wired devices
            if let Some(rate_str) = &polling_rate {
                let rate: u16 = rate_str.parse().unwrap();
                if device.is_wired() && rate > 1000 {
                    return Err(anyhow!(
                        "Wired mouse only supports up to 1000 Hz polling rate."
                    ));
                }
            }

            // Send sensor setting first if present
            if let Some(setting_str) = sensor_setting {
                if let Err(e) = sensor::apply_setting(&device, &setting_str) {
                    eprintln!("{}", e);
                }

                thread::sleep(Duration::from_millis(100));
            }

            // Send debounce setting if present
            if let Some(debounce_str) = debounce {
                if let Err(e) = debounce::apply_setting(&device, &debounce_str) {
                    eprintln!("{}", e);
                }

                thread::sleep(Duration::from_millis(100));
            }

            // Send combined DPI + polling rate packet
            if let Err(e) = performance::apply_settings(&device, dpi_stage, polling_rate.as_deref())
            {
                eprintln!("{}", e);
            }

            // Send sleep timeout setting if present
            if let Some(time) = sleep {
                if let Err(e) = sleep::apply_setting(&device, &time) {
                    eprintln!("{}", e);
                }

                thread::sleep(Duration::from_millis(100));
            }
        }
        Commands::Info(cmd) => match cmd {
            Info::Battery => {
                if let Err(e) = info::get_battery(&device) {
                    eprintln!("Error retrieving battery info: {}", e);
                }
            }
            Info::Sensor => {
                if let Err(e) = info::get_sensor_info(&device) {
                    eprintln!("Error retrieving sensor info: {}", e);
                }
            }
        },
        Commands::Dpi(cmd) => match cmd {
            Dpi::Set {
                stage: _,
                x_dpi,
                y_dpi,
            } => {
                let _ = y_dpi.unwrap_or(x_dpi);
                if let Err(e) = dpi::apply_dpi_setting(&device) {
                    eprintln!("Error setting DPI: {}", e);
                }
            }
        },
    }

    Ok(())
}
