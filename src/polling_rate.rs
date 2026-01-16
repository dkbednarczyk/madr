pub fn get_magic_packet(rate: u16) -> Vec<u8> {
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
