// Sensor settings:
// 0 = basic
// 1 = competitive
// 2 = competitive MAX

pub fn get_magic_packet(sensor_setting: u8) -> Vec<u8> {
    vec![
        0x08,
        0x07,
        0x00,
        0x00,
        0xb5,
        0x06,
        0x00,
        0x55,
        0x06,
        0x4f,
        sensor_setting,                      // sensor setting byte
        0x55u8.wrapping_sub(sensor_setting), // checksum byte
        0x00,
        0x00,
        0x00,
        0x00,
        0x8c,
    ]
}
