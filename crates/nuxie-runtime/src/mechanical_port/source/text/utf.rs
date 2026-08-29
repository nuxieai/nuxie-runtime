pub struct Utf;
impl Utf {
    pub fn count_utf8_length(utf8: &[u8]) -> i32 {
        let mut lead = utf8[0] as u32;
        assert_ne!(lead, 0xff);
        assert_ne!(lead & 0xc0, 0x80);
        if lead & 0x80 == 0 {
            return 1;
        }
        let mut n = 1;
        lead <<= 1;
        while lead & 0x80 != 0 {
            n += 1;
            lead <<= 1;
        }
        assert!((1..=4).contains(&n));
        n
    }
    pub fn next_utf8(utf8: &mut &[u8]) -> u32 {
        let mut n = Self::count_utf8_length(utf8) as usize;
        let first = utf8[0] as u32;
        let mut offset = 1;
        let value = if n == 1 {
            first
        } else {
            let mut value = first & (0xff >> n);
            n -= 1;
            loop {
                value = (value << 6) | (utf8[offset] as u32 & 0x3f);
                offset += 1;
                n -= 1;
                if n == 0 {
                    break value;
                }
            }
        };
        *utf8 = &utf8[offset..];
        value
    }
    pub fn to_utf16(codepoint: u32, output: &mut [u16; 2]) -> i32 {
        if codepoint > 0xffff {
            output[0] = ((0xd800 - 64) | (codepoint >> 10)) as u16;
            output[1] = (0xdc00 | (codepoint & 0x3ff)) as u16;
            2
        } else {
            output[0] = codepoint as u16;
            1
        }
    }
    pub fn count_code_point_length(codepoints: &[u32]) -> u32 {
        codepoints
            .iter()
            .map(|&c| {
                if c <= 0x7f {
                    1
                } else if c <= 0x7ff {
                    2
                } else if c <= 0xffff {
                    3
                } else if c <= 0x10ffff {
                    4
                } else {
                    0
                }
            })
            .sum()
    }
    pub fn encode(output: &mut [u8], c: u32) -> u32 {
        if c <= 0x7f {
            output[0] = c as u8;
            1
        } else if c <= 0x7ff {
            output[0] = (((c >> 6) & 0x1f) | 0xc0) as u8;
            output[1] = ((c & 0x3f) | 0x80) as u8;
            2
        } else if c <= 0xffff {
            output[0] = (((c >> 12) & 0x0f) | 0xe0) as u8;
            output[1] = (((c >> 6) & 0x3f) | 0x80) as u8;
            output[2] = ((c & 0x3f) | 0x80) as u8;
            3
        } else if c <= 0x10ffff {
            output[0] = (((c >> 18) & 7) | 0xf0) as u8;
            output[1] = (((c >> 12) & 0x3f) | 0x80) as u8;
            output[2] = (((c >> 6) & 0x3f) | 0x80) as u8;
            output[3] = ((c & 0x3f) | 0x80) as u8;
            4
        } else {
            0
        }
    }
}
