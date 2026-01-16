mod device;
mod dpi_stage;
mod polling_rate;
mod sensor;

use clap::{Parser, Subcommand, builder::PossibleValuesParser, value_parser};
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
    /// Set device parameters
    Set {
        /// DPI stage to enable (1-6)
        #[arg(short = 'd', long, value_parser = value_parser!(u8).range(1..=6))]
        dpi_stage: Option<u8>,

        #[arg(short = 'p', long, value_parser = PossibleValuesParser::new(["125", "250", "500", "1000", "2000", "4000", "8000"]))]
        polling_rate: Option<String>,

        /// Sensor setting to enable [0=basic, 1=competitive, 2=competitive MAX]
        #[arg(short = 's', long, value_parser = PossibleValuesParser::new(["basic", "competitive", "max"]))]
        sensor_setting: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let device = match Device::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Set {
            dpi_stage,
            polling_rate,
            sensor_setting,
        } => {
            if let Some(stage) = dpi_stage {
                let packet = dpi_stage::get_magic_packet(stage-1);
                match device.send_feature_report(&packet) {
                    Ok(_) => println!("Set DPI stage to {}", stage),
                    Err(e) => eprintln!("Failed to send DPI command: {}", e),
                }
            }

            if let Some(rate_str) = polling_rate {
                let rate: u16 = rate_str.parse().unwrap();
                if device.is_wired() && rate > 1000 {
                    eprintln!("Wired mouse only supports up to 1000 Hz polling rate.");
                    return;
                }

                let packet = polling_rate::get_magic_packet(rate);
                match device.send_feature_report(&packet) {
                    Ok(_) => println!("Set polling rate to {} Hz", rate),
                    Err(e) => eprintln!("Failed to send polling rate command: {}", e),
                }
            }

            if let Some(setting_str) = sensor_setting {
                let setting: u8 = match setting_str.as_str() {
                    "basic" => 0,
                    "competitive" => 1,
                    "max" => 2,
                    _ => unreachable!(),
                };

                let packet = sensor::get_magic_packet(setting);

                match device.send_feature_report(&packet) {
                    Ok(_) => println!("Set sensor setting to {}", setting),
                    Err(e) => eprintln!("Failed to send sensor command: {}", e),
                }
            }
        }
    }
}
