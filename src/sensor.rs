// Sensor settings:
// 0 = basic
// 1 = competitive
// 2 = competitive MAX

use crate::device::Device;

pub fn get_magic_packet(sensor_setting: u8) -> Vec<u8> {
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
pub fn apply_setting(device: &Device, setting_str: &str) -> Result<(), String> {
    let setting: u8 = match setting_str {
        "basic" => 0,
        "competitive" => 1,
        "max" => 2,
        _ => unreachable!(),
    };

    let packet = get_magic_packet(setting);
    device
        .send_feature_report(&packet)
        .map_err(|e| format!("Failed to send sensor command: {}", e))?;

    println!("Set sensor setting to {}", setting_str);
    Ok(())
}
