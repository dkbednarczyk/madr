// Performance settings module
// DPI stage and polling rate share the same packet structure (0x08 0x07 0x00 0x00 0x00 0x06)
// and can be combined into a single configuration packet.

/// Get packet for setting DPI stage only
pub fn get_dpi_packet(dpi_stage: u8) -> Vec<u8> {
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0x00,
        0x06,
        // magic bits for DPI stage
        0x01,
        0x54,
        0x04,
        0x51,
        // set DPI stage index
        dpi_stage,
        0x55u8.wrapping_sub(dpi_stage),
        0x00,
        0x00,
        0x00,
        0x00,
        0x41,
    ]
}

/// Get packet for setting polling rate only
pub fn get_polling_rate_packet(rate: u16) -> Vec<u8> {
    let rate_byte: u8 = match rate {
        125 => 0x08,
        250 => 0x04,
        500 => 0x02,
        1000 => 0x01,
        // wireless only
        2000 => 0x10,
        4000 => 0x20,
        8000 => 0x40,
        _ => unreachable!(),
    };

    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0x00,
        0x06,
        // set polling rate byte
        rate_byte,
        0x55u8.wrapping_sub(rate_byte),
        // magic bits
        0x04,
        0x51,
        0x01,
        0x54,
        0x00,
        0x00,
        0x00,
        0x00,
        0x41,
    ]
}

/// Get combined packet for setting both DPI stage and polling rate
/// This is more reliable than sending two separate packets as they share the same command structure
pub fn get_combined_packet(dpi_stage: u8, rate: u16) -> Vec<u8> {
    let rate_byte: u8 = match rate {
        125 => 0x08,
        250 => 0x04,
        500 => 0x02,
        1000 => 0x01,
        2000 => 0x10,
        4000 => 0x20,
        8000 => 0x40,
        _ => unreachable!(),
    };

    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0x00,
        0x06,
        rate_byte,                      // byte 6: polling rate
        0x55u8.wrapping_sub(rate_byte), // byte 7: polling rate checksum
        0x04,
        0x51,                           // bytes 8-9: magic bits
        dpi_stage,                      // byte 10: DPI stage
        0x55u8.wrapping_sub(dpi_stage), // byte 11: DPI stage checksum
        0x00,
        0x00,
        0x00,
        0x00,
        0x41, // bytes 12-16: trailer
    ]
}

/// Build the appropriate packet based on which settings are provided
pub fn build_packet(dpi_stage: Option<u8>, polling_rate: Option<u16>) -> Option<Vec<u8>> {
    match (dpi_stage, polling_rate) {
        (Some(stage), Some(rate)) => Some(get_combined_packet(stage, rate)),
        (Some(stage), None) => Some(get_dpi_packet(stage)),
        (None, Some(rate)) => Some(get_polling_rate_packet(rate)),
        (None, None) => None,
    }
}
