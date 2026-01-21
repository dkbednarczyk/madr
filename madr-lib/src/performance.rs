// Performance settings module
// DPI stage and polling rate share the same report structure (0x08 0x07 0x00 0x00 0x00 0x06)
// and can be combined into a single configuration report.

use crate::device::Device;
use crate::{MadRError, Result};

pub struct Performance {
    dpi_stage: u8,
    polling_rate: u16,
}

impl Performance {
    pub fn new(dpi_stage: u8, polling_rate: u16) -> Self {
        Self {
            dpi_stage,
            polling_rate,
        }
    }

    pub fn dpi_stage(&self) -> u8 {
        self.dpi_stage
    }

    pub fn polling_rate(&self) -> u16 {
        self.polling_rate
    }

    fn from_bytes(data: &[u8]) -> Result<Performance> {
        println!("Parsing performance data: {:x?}", data);
        let dpi_stage = data[10] + 1; // stored as stage - 1
        let polling_rate = match data[6] {
            0x08 => 125,
            0x04 => 250,
            0x02 => 500,
            0x01 => 1000,
            0x10 => 2000,
            0x20 => 4000,
            0x40 => 8000,
            _ => {
                return Err(MadRError::InvalidPerformanceSetting(
                    "Unsupported polling rate".into(),
                ));
            }
        };

        Ok(Self {
            dpi_stage,
            polling_rate,
        })
    }
}

fn make_combined_report(dpi_stage: u8, rate: u16) -> Result<Vec<u8>> {
    let rate_byte: u8 = match rate {
        125 => 0x08,
        250 => 0x04,
        500 => 0x02,
        1000 => 0x01,
        // wireless only
        2000 => 0x10,
        4000 => 0x20,
        8000 => 0x40,
        _ => {
            return Err(MadRError::InvalidPerformanceSetting(
                "Unsupported polling rate".into(),
            ))?;
        }
    };

    Ok(vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0x00,
        0x06,
        rate_byte,                      // byte 6: polling rate
        0x55u8.wrapping_sub(rate_byte), // byte 7: polling rate checksum
        0x04,
        0x51,                               // bytes 8-9: magic bits
        dpi_stage - 1,                      // byte 10: DPI stage
        0x55u8.wrapping_sub(dpi_stage - 1), // byte 11: DPI stage checksum
        0x00,
        0x00,
        0x00,
        0x00,
        0x41, // bytes 12-16: trailer
    ])
}

pub fn get_settings(device: &Device) -> Result<Performance> {
    let mut report = [0u8; 17];
    report[0] = 0x08;
    report[1] = 0x08;
    report[5] = 0x06;
    report[16] = 0x3f;

    device.write(&report)?;

    let mut buf = [0u8; 17];
    device.read_timeout(&mut buf, 20)?;

    println!("Performance report data: {:x?}", &buf);

    Performance::from_bytes(&buf)
}

/// Apply performance settings to device
pub fn apply_settings(device: &Device, settings: &Performance) -> Result<()> {
    let report = make_combined_report(settings.dpi_stage, settings.polling_rate)?;
    device.send_feature_report(&report)?;

    Ok(())
}
