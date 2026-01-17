use std::{thread, time::Duration};

use crate::device::Device;
use anyhow::Result;

pub struct DpiStage {
    pub x_dpi: u16,
    pub y_dpi: u16,
    pub rgb: (u8, u8, u8),
}

impl DpiStage {
    pub fn new(x_dpi: u16, y_dpi: u16, rgb: (u8, u8, u8)) -> Self {
        Self { x_dpi, y_dpi, rgb }
    }

    pub fn symmetric(dpi: u16, rgb: (u8, u8, u8)) -> Self {
        Self::new(dpi, dpi, rgb)
    }
}

pub fn get_dpi_packet_pair(packet_index: u8, x_a: u16, y_a: u16, x_b: u16, y_b: u16) -> Vec<u8> {
    let packet_id = 0x04 + (packet_index * 0x08);

    let encode_dpi = |x: u16, y: u16| -> (u8, u8, u8, u8) {
        // Mouse stores DPI in units of 50 DPI, with an offset of 1
        // i.e., 50 DPI = 0, 100 DPI = 1, etc.
        // let's assume we have x and y as 30000, the max
        let x_val = (x / 50).saturating_sub(1);
        let y_val = (y / 50).saturating_sub(1);

        // at this point x_val and y_val are both larger than 255, max u8.
        // so first we get only lowest 8 bits of each value
        let x_low = (x_val & 0xFF) as u8;
        let y_low = (y_val & 0xFF) as u8;

        // and now the highest 8 bits (anything above 255)
        // there should be max 2 bits here
        let x_high = ((x_val >> 8) & 0xFF) as u8;
        let y_high = ((y_val >> 8) & 0xFF) as u8;

        // so what we have done is
        // 30000 dpi -> 599 -> 00000010 01010111
        // low  byte -> 01010111 (87)
        // high byte -> 00000010 (2)

        // we pack the high bits into a single byte
        // y_high << 6 = 10000000
        // x_high << 2 = 00001000
        // OR them together = 136
        let high_container = (y_high << 6) | (x_high << 2);

        // Calculate checksum
        let checksum = 0x55u8
            .wrapping_sub(x_low)
            .wrapping_sub(y_low)
            .wrapping_sub(high_container);

        (x_low, y_low, high_container, checksum)
    };

    let (x_low_a, y_low_a, high_a, check_a) = encode_dpi(x_a, y_a);
    let (x_low_b, y_low_b, high_b, check_b) = encode_dpi(x_b, y_b);

    // See documentation/dpi-and-rgb-encoding.md for packet structure
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        packet_id,
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
        0x94u8.wrapping_sub(packet_id),
    ]
}

pub fn get_rgb_packet_pair(packet_index: u8, rgb_a: (u8, u8, u8), rgb_b: (u8, u8, u8)) -> Vec<u8> {
    let packet_id = 0x24 + (packet_index * 0x08);

    let checksum_a = 0x55u8
        .wrapping_sub(rgb_a.0)
        .wrapping_sub(rgb_a.1)
        .wrapping_sub(rgb_a.2);
    let checksum_b = 0x55u8
        .wrapping_sub(rgb_b.0)
        .wrapping_sub(rgb_b.1)
        .wrapping_sub(rgb_b.2);

    // See documentation/dpi-and-rgb-encoding.md for packet structure
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        packet_id,
        0x08,
        rgb_a.0,
        rgb_a.1,
        rgb_a.2,
        checksum_a,
        rgb_b.0,
        rgb_b.1,
        rgb_b.2,
        checksum_b,
        0x00,
        0x00,
        0x94u8.wrapping_sub(packet_id),
    ]
}

pub fn get_all_stage_packets(stages: &[DpiStage; 4]) -> Vec<Vec<u8>> {
    vec![
        // DPI packets (two stages per packet)
        get_dpi_packet_pair(
            1,
            stages[0].x_dpi,
            stages[0].y_dpi,
            stages[1].x_dpi,
            stages[1].y_dpi,
        ),
        get_dpi_packet_pair(
            2,
            stages[2].x_dpi,
            stages[2].y_dpi,
            stages[3].x_dpi,
            stages[3].y_dpi,
        ),
        // RGB packets (two stages per packet)
        get_rgb_packet_pair(1, stages[0].rgb, stages[1].rgb),
        get_rgb_packet_pair(2, stages[2].rgb, stages[3].rgb),
    ]
}

pub fn apply_dpi_setting(device: &Device) -> Result<()> {
    let stages = [
        DpiStage::symmetric(1600, (255, 0, 0)),
        DpiStage::symmetric(1600, (0, 255, 0)),
        DpiStage::new(1600, 3200, (0, 0, 255)),
        DpiStage::symmetric(1600, (255, 0, 255)),
    ];

    let packets = get_all_stage_packets(&stages);
    for packet in packets {
        println!("Sending packet: {:02x?}", packet);
        device.send_feature_report(&packet)?;
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
