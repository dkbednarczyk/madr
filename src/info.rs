use anyhow::anyhow;
use anyhow::Result;
use colored::Colorize;

use crate::device::Device;

/*
when the web app is loaded, the first page is a summary of the device info
including: model name (and/or some other data that identifies the device), battery status, connection type (wired/2.4g)

there are a total of 5 packets sent to get this info
in order:

unsigned char bytes[] = {0x8, 0x1, 0x0, 0x0, 0x0, 0x8, 0x56, 0x85, 0x33, 0x5d, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xd9};

these two should also return some kind of info about the device as they follow the latter battery info pattern
where the sum of all bytes equals 0x55
unsigned char bytes[] = {0x8, 0x12, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3b};
unsigned char bytes[] = {0x8, 0x3, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4a};

this packet returns battery info
unsigned char bytes[] = {0x8, 0x4, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x49};
example response: [08, 04, 00, 00, 00, 02, 5F, 01, 10, 44, 00, 00, 00, 00, 00, 00, 93]
- Byte 6 (0x5F): Battery Percentage.
- Byte 7 (00 or 01): Charging Status.
- Bytes 8-9 (1044): Voltage.
    - Wireless: 0x0FF8 = 4088 mV (4.08V).
    - Wired: 0x1044 = 4164 mV (4.16V).

unsigned char bytes[] = {0x8, 0x1, 0x0, 0x0, 0x0, 0x8, 0xdc, 0xb9, 0x23, 0x88, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4};
*/

#[derive(Debug)]
pub struct BatteryStatus {
    percentage: u8,
    voltage_mv: u16,
    is_charging: bool,
}

#[derive(Debug)]
pub struct SensorInfo {
    setting: u8,
}

impl SensorInfo {
    pub fn setting_name(&self) -> &str {
        match self.setting {
            0 => "basic",
            1 => "competitive",
            2 => "max",
            _ => "unknown",
        }
    }
}

fn parse_battery_report(data: &[u8]) -> Option<BatteryStatus> {
    // Validate Header (0x08), Command (0x04), and Length
    if data.len() < 17 || data[0] != 0x08 || data[1] != 0x04 {
        return None;
    }

    // Byte 6: Percentage (0-100)
    let percentage = data[6];

    // Byte 7: Charging Flag (0x00 = Wireless, 0x01 = Wired/Charging)
    let is_charging = data[7] == 0x01;

    // Bytes 8-9: Voltage (Big Endian)
    let voltage_mv = u16::from_be_bytes([data[8], data[9]]);

    Some(BatteryStatus {
        percentage,
        voltage_mv,
        is_charging,
    })
}

fn parse_sensor_report(data: &[u8]) -> Option<SensorInfo> {
    // Validate Header (0x08), Command (0x08), and Length
    if data.len() < 17 || data[0] != 0x08 || data[1] != 0x08 {
        return None;
    }

    Some(SensorInfo { setting: data[10] })
}

pub fn get_battery(device: &Device) -> Result<()> {
    let mut packet = [0u8; 17]; // Assuming 17 bytes based on your previous array
    packet[0] = 0x08;
    packet[1] = 0x04;
    packet[16] = 0x55 - (0x08 + packet[1]);

    match device.write(&packet) {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Failed to send battery request: {}", e)),
    }

    let mut buf = [0u8; 256];
    let size = if let Ok(s) = device.read_timeout(&mut buf, 20) {
        s
    } else {
        return Err(anyhow!("Failed to read battery response"));
    };

    let data = &buf[..size];

    if let Some(status) = parse_battery_report(data) {
        let percentage_str = match status.percentage {
            81..=100 => format!("{}%", status.percentage).bright_green(),
            51..=80 => format!("{}%", status.percentage).green(),
            31..=50 => format!("{}%", status.percentage).yellow(),
            16..=30 => format!("{}%", status.percentage).bright_red(),
            _ => format!("{}%", status.percentage).red().bold(),
        };

        let voltage_v = status.voltage_mv as f32 / 1000.0;

        let status_str = if status.is_charging {
            "Charging".bright_cyan().bold()
        } else {
            "Not Charging".bright_white()
        };

        println!("Battery: {percentage_str} | {voltage_v:.2}V | {status_str}",);

        if !status.is_charging && device.is_wired() {
            println!(
                "{}",
                "warning: device is wired but not charging".red().bold()
            );
        }
    } else {
        return Err(anyhow!("Invalid battery report format"));
    }

    Ok(())
}

pub fn get_sensor_info(device: &Device) -> Result<SensorInfo> {
    let mut packet = [0u8; 17];
    packet[0] = 0x08;
    packet[1] = 0x08;
    packet[4] = 0xB5;
    packet[5] = 0x06;
    packet[16] = 0x8a;

    match device.write(&packet) {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Failed to send sensor request: {}", e)),
    }

    let mut buf = [0u8; 256];
    let size = if let Ok(s) = device.read_timeout(&mut buf, 20) {
        s
    } else {
        return Err(anyhow!("Failed to read sensor response"));
    };

    let data = &buf[..size];

    if let Some(info) = parse_sensor_report(data) {
        println!("{}", info.setting_name());
        Ok(info)
    } else {
        Err(anyhow!("Invalid sensor report format"))
    }
}
