pub fn get_magic_packet(dpi_stage: u8) -> Vec<u8> {
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
