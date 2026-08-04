#[allow(non_snake_case)]
pub(crate) fn to_utf_8(data: &mut [u8], code: u32) -> usize {
    // U+0000..U+007F
    if code < 0x80 {
        data[0] = code as u8;
        1
    }
    // U+0080..U+07FF
    else if code < 0x800 {
        data[0] = (0xC0 | (code >> 6)) as u8;
        data[1] = (0x80 | (code & 0x3F)) as u8;
        2
    }
    // U+0800..U+FFFF
    else if code < 0x10000 {
        data[0] = (0xE0 | (code >> 12)) as u8;
        data[1] = (0x80 | ((code >> 6) & 0x3F)) as u8;
        data[2] = (0x80 | (code & 0x3F)) as u8;
        3
    }
    // U+10000..U+10FFFF
    else if code < 0x110000 {
        data[0] = (0xF0 | (code >> 18)) as u8;
        data[1] = (0x80 | ((code >> 12) & 0x3F)) as u8;
        data[2] = (0x80 | ((code >> 6) & 0x3F)) as u8;
        data[3] = (0x80 | (code & 0x3F)) as u8;
        4
    } else {
        0
    }
}
