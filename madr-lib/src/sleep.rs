use crate::device::Device;
use crate::{MadRError, Result};
use std::thread;
use std::time::Duration;

pub fn get_sleep_report(tens_of_seconds: u8) -> Vec<u8> {
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0xA9,
        0x0A,
        0x04,
        0x51,
        0x01,
        0x54,
        tens_of_seconds,
        0x55u8.wrapping_sub(tens_of_seconds),
        0x00,
        0x55,
        0x00,
        0x55,
        0xEA,
    ]
}

pub fn get_confirmation_report(tens_of_seconds: u8) -> Vec<u8> {
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0xB5,
        0x06,
        0x01,
        0x54,
        tens_of_seconds,
        0x55u8.wrapping_sub(tens_of_seconds),
        0x00,
        0x55,
        0x00,
        0x00,
        0x00,
        0x00,
        0x8C,
    ]
}

/// Apply sleep timeout setting to device
pub fn apply_setting(device: &Device, time_str: &str) -> Result<()> {
    let tens_of_seconds: u8 = match time_str {
        "30s" => 3,
        "1m" => 6,
        "2m" => 12,
        "3m" => 18,
        "5m" => 30,
        "20m" => 120,
        "25m" => 150,
        "30m" => 180,
        _ => return Err(MadRError::InvalidSleepTimeout(time_str.into())),
    };

    let sleep_pkt = get_sleep_report(tens_of_seconds);
    device.send_feature_report(&sleep_pkt)?;

    // Device protocol requires a delay between the two reports
    thread::sleep(Duration::from_millis(50));

    let confirmation = get_confirmation_report(tens_of_seconds);
    device.send_feature_report(&confirmation)?;

    Ok(())
}
