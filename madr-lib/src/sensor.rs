use crate::device::Device;
use crate::{MadRError, Result};

#[derive(Debug)]
pub struct Sensor {
    pub setting: u8,
}

impl Sensor {
    pub fn from_bytes(data: &[u8]) -> Option<Sensor> {
        if data.len() < 17 || data[0] != 0x08 || data[1] != 0x08 {
            return None;
        }

        Some(Self { setting: data[10] })
    }

    pub fn name_from_setting(&self) -> &str {
        match self.setting {
            0 => "basic",
            1 => "competitive",
            2 => "max",
            _ => "unknown",
        }
    }

    pub fn setting_from_name(name: &str) -> Option<u8> {
        match name {
            "basic" => Some(0),
            "competitive" => Some(1),
            "max" => Some(2),
            _ => None,
        }
    }
}

pub fn get_sensor_info(device: &Device) -> Result<Sensor> {
    let mut report = [0u8; 17];
    report[0] = 0x08;
    report[1] = 0x08;
    report[4] = 0xB5;
    report[5] = 0x06;
    report[16] = 0x8a;

    device.write(&report)?;

    let mut buf = [0u8; 17];
    device.read_timeout(&mut buf, 20)?;

    Sensor::from_bytes(&buf).ok_or(MadRError::InvalidSensorFormat)
}

pub fn get_magic_report(sensor_setting: u8) -> Vec<u8> {
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0xb5,                                // magic
        0x06,                                // bytes
        0x00,           // works with either 00 or 01? after factory reset 00 is correct though
        0x55,           // 55 - prev
        0x06,           // magic
        0x4f,           // magic
        sensor_setting, // sensor setting byte
        0x55u8.wrapping_sub(sensor_setting), // checksum byte
        0x00,
        0x00,
        0x00,
        0x00,
        0x8c,
    ]
}

/// Apply sensor setting to device
pub fn apply_setting(device: &Device, setting_str: &str) -> Result<()> {
    let setting = Sensor::setting_from_name(setting_str)
        .ok_or_else(|| MadRError::InvalidSensorSetting(setting_str.into()))?;

    let report = get_magic_report(setting);
    device.send_feature_report(&report)?;

    Ok(())
}
