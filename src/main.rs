mod debounce;
mod device;
mod performance;
mod sensor;
mod sleep;

use clap::{builder::PossibleValuesParser, value_parser, Parser, Subcommand};
use device::Device;
use std::thread;
use std::time::Duration;

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
    #[command(arg_required_else_help = true)]
    Set {
        /// DPI stage to enable
        #[arg(short = 'd', long, value_parser = value_parser!(u8).range(1..=6))]
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
            debounce,
            sleep,
        } => {
            // Validate polling rate for wired devices
            if let Some(rate_str) = &polling_rate {
                let rate: u16 = rate_str.parse().unwrap();
                if device.is_wired() && rate > 1000 {
                    eprintln!("Wired mouse only supports up to 1000 Hz polling rate.");
                    return;
                }
            }

            // Send sensor setting first if present
            if let Some(setting_str) = sensor_setting {
                let setting: u8 = match setting_str.as_str() {
                    "basic" => 0,
                    "competitive" => 1,
                    "max" => 2,
                    _ => unreachable!(),
                };

                let packet = sensor::get_magic_packet(setting);
                match device.send_feature_report(&packet) {
                    Ok(_) => {
                        println!("Set sensor setting to {}", setting_str);
                        thread::sleep(Duration::from_millis(200));
                    }
                    Err(e) => eprintln!("Failed to send sensor command: {}", e),
                }
            }

            // Send debounce setting if present
            if let Some(debounce_str) = debounce {
                let debounce_val: u8 = debounce_str.parse().unwrap();
                if let 0..=2 = debounce_val {
                    eprintln!("Debounce times under 4 ms are not recommended.");
                }

                let packet = debounce::get_debounce_packet(debounce_val);
                match device.send_feature_report(&packet) {
                    Ok(_) => {
                        println!("Set debounce time to {} ms", debounce_val);
                        thread::sleep(Duration::from_millis(200));
                    }
                    Err(e) => eprintln!("Failed to send debounce command: {}", e),
                }
            }

            // Send combined DPI + polling rate packet last
            let polling_rate_val = polling_rate.as_ref().map(|s| s.parse::<u16>().unwrap());
            if let Some(packet) = performance::build_packet(dpi_stage, polling_rate_val) {
                match device.send_feature_report(&packet) {
                    Ok(_) => {
                        if let Some(stage) = dpi_stage {
                            println!("Set DPI stage to {}", stage);
                        }
                        if let Some(rate) = polling_rate_val {
                            println!("Set polling rate to {} Hz", rate);
                        }
                    }
                    Err(e) => eprintln!("Failed to send configuration command: {}", e),
                }
            }

            // Send sleep timeout setting if present
            if let Some(time) = sleep {
                let tens_of_seconds: u8 = match time.as_str() {
                    "30s" => 3,
                    "1m" => 6,
                    "2m" => 12,
                    "3m" => 18,
                    "5m" => 30,
                    "20m" => 120,
                    "25m" => 150,
                    "30m" => 180,
                    _ => unreachable!(),
                };

                let packet1 = sleep::get_sleep_packet(tens_of_seconds);
                match device.send_feature_report(&packet1) {
                    Ok(_) => {
                        thread::sleep(Duration::from_millis(200));
                        let packet2 = sleep::get_second_packet(tens_of_seconds);
                        match device.send_feature_report(&packet2) {
                            Ok(_) => println!("Set sleep timeout to {}", time),
                            Err(e) => eprintln!("Failed to send sleep confirmation: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to send sleep command: {}", e),
                }
            }
        }
    }
}