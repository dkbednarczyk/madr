use anyhow::anyhow;
use anyhow::Result;

use clap::{builder::PossibleValuesParser, value_parser, Parser, Subcommand};

use madr_lib::{
    battery, debounce,
    device::Device,
    dpi,
    performance::{self, Performance},
    sensor, sleep,
};

#[derive(Parser)]
#[command(name = "madrctl")]
#[command(about = "Control your VXE MAD R series gaming mouse from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Change debounce settings
    #[clap(subcommand)]
    Debounce(Debounce),

    /// Change sleep timeout settings
    #[clap(subcommand)]
    Sleep(Sleep),

    /// Change dpi settings
    #[clap(subcommand)]
    Dpi(Dpi),

    /// Change polling rate settings
    #[clap(subcommand)]
    Polling(Polling),

    /// Change sensor settings
    #[clap(subcommand)]
    Sensor(Sensor),

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
enum Debounce {
    /// Set debounce time
    Set {
        /// Debounce time in milliseconds
        #[arg(value_parser = PossibleValuesParser::new(["0", "1", "2", "4", "8", "15", "20"]))]
        time: String,
    },
}

#[derive(Subcommand)]
enum Sleep {
    /// Set sleep timeout
    Set {
        /// Sleep timeout (inactivity before sleep)
        #[arg(value_parser = PossibleValuesParser::new(["30s", "1m", "2m", "3m", "5m", "20m", "25m", "30m"]))]
        timeout: String,
    },
}


#[derive(Subcommand)]
enum Dpi {
    SetStage {
        /// DPI stage to set active (1-8)
        #[arg(value_parser = value_parser!(u8).range(1..=8))]
        stage: u8
    },

    /// Change DPI settings for a specific stage
    ModifyStage {
        /// DPI stage to change (1-8)
        #[arg(short, long, value_parser = value_parser!(u8).range(1..=8))]
        stage: u8,
        /// X DPI value
        #[arg(short, long, value_parser = value_parser!(u16).range(50..=16000))]
        x_dpi: Option<u16>,
        /// Y DPI value, if not specified, X DPI will be used
        #[arg(short, long, value_parser = value_parser!(u16).range(50..=16000))]
        y_dpi: Option<u16>,
        /// RGB color in 255,255,255 format, if not specified, color will not be changed
        #[arg(short, long)]
        rgb: Option<String>,
    },
}

#[derive(Subcommand)]
enum Polling {
    /// Set polling rate
    Set {
        /// Polling rate in Hz
        #[arg(value_parser = PossibleValuesParser::new(["125", "250", "500", "1000", "2000", "4000", "8000"]))]
        polling_rate: String,
    },
}

#[derive(Subcommand)]
enum Sensor {
    /// Set sensor preset
    Set {
        /// Preset to apply
        #[arg(value_parser = PossibleValuesParser::new(["basic", "competitive", "max"]))]
        preset: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let device = Device::open()?;

    match cli.command {
        Commands::Debounce(cmd) => match cmd {
            Debounce::Set { time } => {
                match time.as_str() {
                    "0" | "1" | "2" => {
                        eprintln!("warning: low debounce values are not recommended")
                    }
                    _ => (),
                }
                debounce::apply_setting(&device, &time)?;
            }
        },
        Commands::Sleep(cmd) => match cmd {
            Sleep::Set { timeout } => {
                sleep::apply_setting(&device, &timeout)?;
            }
        },
        Commands::Info(cmd) => match cmd {
            Info::Battery => {
                let b = battery::get_battery_info(&device)?;
                println!("{:?}", b);
            }
            Info::Sensor => {
                let s = sensor::get_sensor_info(&device)?;
                println!("{:?}", s);
            }
        },
        Commands::Dpi(cmd) => match cmd {
            Dpi::SetStage { stage } => {
                let settings = performance::get_settings(&device)?;
                performance::apply_settings(&device, &Performance::new(stage, settings.polling_rate()))?;
            }
            Dpi::ModifyStage {
                stage,
                x_dpi,
                y_dpi,
                rgb,
            } => {
                dpi::apply_dpi_setting(&device, stage, x_dpi, y_dpi, rgb.as_deref())?;
            }
        },
        Commands::Polling(cmd) => match cmd {
            Polling::Set { polling_rate } => {
                let rate: u16 = polling_rate.parse().unwrap();

                // Validate polling rate for wired devices
                if device.is_wired() && rate > 1000 {
                    return Err(anyhow!(
                        "Wired mouse only supports up to 1000 Hz polling rate."
                    ));
                }

                let settings = performance::get_settings(&device)?;
                performance::apply_settings(&device, &Performance::new(settings.dpi_stage(), rate))?;
            }
        },
        Commands::Sensor(cmd) => match cmd {
            Sensor::Set { preset } => {
                sensor::apply_setting(&device, &preset)?;
            }
        },
    }

    Ok(())
}
