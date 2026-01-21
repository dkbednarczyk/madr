// See documentation/dpi-and-rgb-encoding.md for details on encoding

use std::str::FromStr;

use crate::device::Device;
use crate::{MadRError, Result};

pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl FromStr for Rgb {
    type Err = MadRError;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 3 {
            return Err(MadRError::InvalidRgb(
                "Invalid RGB format. Expected format: R,G,B".into(),
            ));
        }

        let r: u8 = parts[0]
            .parse()
            .map_err(|_| MadRError::InvalidRgb("Invalid R value".into()))?;
        let g: u8 = parts[1]
            .parse()
            .map_err(|_| MadRError::InvalidRgb("Invalid G value".into()))?;
        let b: u8 = parts[2]
            .parse()
            .map_err(|_| MadRError::InvalidRgb("Invalid B value".into()))?;

        Ok(Rgb { r, g, b })
    }
}

pub struct DpiStage {
    x_dpi: u16,
    y_dpi: u16,
}

impl DpiStage {
    pub fn new(x_dpi: u16, y_dpi: u16) -> Self {
        Self { x_dpi, y_dpi }
    }
}

pub fn read_dpi_stages(device: &Device, report_index: u8) -> Result<Vec<u8>> {
    let report_id = 0x04 + (report_index * 0x08);

    let mut request = vec![0u8; 17];
    request[0] = 0x08;
    request[1] = 0x08;
    request[4] = report_id;
    request[5] = 0x08;
    request[16] = 0x3Du8.wrapping_sub(report_id);

    device.send_feature_report(&request)?;

    let mut buf = [0u8; 17];
    device.read_timeout(&mut buf, 20)?;

    Ok(buf.to_vec())
}

pub fn decode_dpi_pair(report: &[u8]) -> (DpiStage, DpiStage) {
    let decode_dpi = |x_low: u8, y_low: u8, high_container: u8| -> (u16, u16) {
        let x_high = (high_container >> 2) & 0x0F;
        let y_high = (high_container >> 6) & 0x03;

        let x_val = ((x_high as u16) << 8) | (x_low as u16);
        let y_val = ((y_high as u16) << 8) | (y_low as u16);

        let x_dpi = (x_val + 1) * 50;
        let y_dpi = (y_val + 1) * 50;

        (x_dpi, y_dpi)
    };

    let (x_dpi_a, y_dpi_a) = decode_dpi(report[6], report[7], report[8]);
    let (x_dpi_b, y_dpi_b) = decode_dpi(report[10], report[11], report[12]);

    let stage_a = DpiStage::new(x_dpi_a, y_dpi_a);
    let stage_b = DpiStage::new(x_dpi_b, y_dpi_b);

    (stage_a, stage_b)
}

pub fn read_rgb_stages(device: &Device, report_index: u8) -> Result<Vec<u8>> {
    let report_id = 0x24 + (report_index * 0x08);

    let mut request = vec![0u8; 17];
    request[0] = 0x08;
    request[1] = 0x08;
    request[4] = report_id;
    request[5] = 0x08;
    request[16] = 0x3Du8.wrapping_sub(report_id);

    device.send_feature_report(&request)?;

    let mut buf = [0u8; 17];
    device.read_timeout(&mut buf, 20)?;

    Ok(buf.to_vec())
}

pub fn decode_rgb_pair(response: &[u8]) -> (Rgb, Rgb) {
    let decode = |offset: usize| -> Rgb {
        Rgb {
            r: response[offset],
            g: response[offset + 1],
            b: response[offset + 2],
        }
    };

    (decode(6), decode(10))
}

pub fn encode_dpi_pair(report_index: u8, stage_a: &DpiStage, stage_b: &DpiStage) -> Vec<u8> {
    let report_id = 0x04 + (report_index * 0x08);

    let encode_dpi = |x: u16, y: u16| -> (u8, u8, u8, u8) {
        let x_val = (x / 50).saturating_sub(1);
        let y_val = (y / 50).saturating_sub(1);

        let x_low = (x_val & 0xFF) as u8;
        let y_low = (y_val & 0xFF) as u8;

        let x_high = ((x_val >> 8) & 0xFF) as u8;
        let y_high = ((y_val >> 8) & 0xFF) as u8;

        let high_container = (y_high << 6) | (x_high << 2);

        let checksum = 0x55u8
            .wrapping_sub(x_low)
            .wrapping_sub(y_low)
            .wrapping_sub(high_container);

        (x_low, y_low, high_container, checksum)
    };

    let (x_low_a, y_low_a, high_a, check_a) = encode_dpi(stage_a.x_dpi, stage_a.y_dpi);
    let (x_low_b, y_low_b, high_b, check_b) = encode_dpi(stage_b.x_dpi, stage_b.y_dpi);

    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        report_id,
        0x08,
        x_low_a,
        y_low_a,
        high_a,
        check_a,
        x_low_b,
        y_low_b,
        high_b,
        check_b,
        0x00,
        0x00,
        0x94u8.wrapping_sub(report_id),
    ]
}

pub fn encode_rgb_pair(report_index: u8, rgb_a: &Rgb, rgb_b: &Rgb) -> Vec<u8> {
    let report_id = 0x24 + (report_index * 0x08);

    let checksum_a = 0x55u8
        .wrapping_sub(rgb_a.r)
        .wrapping_sub(rgb_a.g)
        .wrapping_sub(rgb_a.b);
    let checksum_b = 0x55u8
        .wrapping_sub(rgb_b.r)
        .wrapping_sub(rgb_b.g)
        .wrapping_sub(rgb_b.b);

    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        report_id,
        0x08,
        rgb_a.r,
        rgb_a.g,
        rgb_a.b,
        checksum_a,
        rgb_b.r,
        rgb_b.g,
        rgb_b.b,
        checksum_b,
        0x00,
        0x00,
        0x94u8.wrapping_sub(report_id),
    ]
}

pub fn apply_dpi_setting(
    device: &Device,
    stage: u8,
    x_dpi: Option<u16>,
    y_dpi: Option<u16>,
    rgb: Option<&str>,
) -> Result<()> {
    if x_dpi.is_none() && rgb.is_none() {
        return Err(MadRError::InvalidDpi(
            "At least one of X DPI or RGB must be specified".into(),
        ));
    }

    let report_index: u8 = (stage as f32 / 2.0).ceil() as u8;

    if let Some(x_dpi_val) = x_dpi {
        if x_dpi_val % 50 != 0 || x_dpi_val < 100 || x_dpi_val > 30000 {
            return Err(MadRError::InvalidDpi(
                "X DPI must be between 100 and 30000 and a multiple of 50".into(),
            ));
        }

        if let Some(y_dpi_val) = y_dpi {
            if y_dpi_val % 50 != 0 || y_dpi_val < 100 || y_dpi_val > 30000 {
                return Err(MadRError::InvalidDpi(
                    "Y DPI must be between 100 and 30000 and a multiple of 50".into(),
                ));
            }
        }

        let dpi_stages = read_dpi_stages(device, report_index)?;
        let (mut stage_a, mut stage_b) = decode_dpi_pair(&dpi_stages);

        if stage % 2 == 1 {
            stage_a.x_dpi = x_dpi_val;
            stage_a.y_dpi = y_dpi.unwrap_or(x_dpi_val);
        } else {
            stage_b.x_dpi = x_dpi_val;
            stage_b.y_dpi = y_dpi.unwrap_or(x_dpi_val);
        }

        let dpi_report = encode_dpi_pair(report_index, &stage_a, &stage_b);
        device.send_feature_report(&dpi_report)?;
    }

    if let Some(rgb_str) = rgb {
        let parsed = Rgb::from_str(rgb_str)?;

        let rgb_stages = read_rgb_stages(device, report_index)?;
        let (mut rgb_a, mut rgb_b) = decode_rgb_pair(&rgb_stages);

        if stage % 2 == 1 {
            rgb_a = parsed;
        } else {
            rgb_b = parsed;
        }

        let rgb_report = encode_rgb_pair(report_index, &rgb_a, &rgb_b);
        device.send_feature_report(&rgb_report)?;
    };

    Ok(())
}
