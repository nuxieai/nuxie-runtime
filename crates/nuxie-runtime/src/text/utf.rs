/// Direct owner for pinned `UTF::CountUTF8Length` at Rust's checked-slice
/// boundary. C++ validates only the lead byte and deliberately does not
/// inspect continuation bytes.
pub(crate) fn count_utf8_length(utf8: &[u8]) -> Option<usize> {
    match *utf8.first()? {
        0x00..=0x7f => Some(1),
        0xc0..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf7 => Some(4),
        // Pinned C++ asserts for a continuation, 0xf8..=0xfe, or 0xff lead.
        _ => None,
    }
}

/// Direct owner for pinned `UTF::NextUTF8`.
///
/// The source accepts overlong encodings, surrogate encodings, and arbitrary
/// low six bits in continuation positions. Rust preserves those byte-level
/// semantics while rejecting a truncated slice instead of reading past it.
pub(crate) fn next_utf8(utf8: &[u8], offset: &mut usize) -> Option<u32> {
    let remaining = utf8.get(*offset..)?;
    let width = count_utf8_length(remaining)?;
    let sequence = remaining.get(..width)?;
    let first = u32::from(sequence[0]);
    let mut value = if width == 1 {
        first
    } else {
        first & (0xffu32 >> width)
    };
    for byte in &sequence[1..] {
        value = (value << 6) | (u32::from(*byte) & 0x3f);
    }
    *offset = offset.checked_add(width)?;
    Some(value)
}

/// Direct owner for pinned `UTF::ToUTF16`; returns the number of written
/// 16-bit values. The source accepts every `Unichar`, including values above
/// the Unicode maximum, and narrows the same intermediate values to u16.
pub(crate) fn to_utf16(unichar: u32, utf16: &mut [u16; 2]) -> usize {
    if unichar > 0xffff {
        utf16[0] = ((0xd800u32 - 64) | (unichar >> 10)) as u16;
        utf16[1] = (0xdc00u32 | (unichar & 0x3ff)) as u16;
        2
    } else {
        utf16[0] = unichar as u16;
        1
    }
}

/// Direct owner for pinned `UTF::CountCodePointLength`. Values above the
/// Unicode maximum contribute zero, and the unsigned accumulator wraps.
pub(crate) fn count_code_point_length(codepoints: &[u32]) -> u32 {
    codepoints.iter().fold(0u32, |length, codepoint| {
        let encoded_length = match *codepoint {
            0x0000..=0x007f => 1,
            0x0080..=0x07ff => 2,
            0x0800..=0xffff => 3,
            0x1_0000..=0x10_ffff => 4,
            _ => 0,
        };
        length.wrapping_add(encoded_length)
    })
}

/// Direct owner for pinned `UTF::Encode`; returns the number of bytes written
/// and leaves the output untouched when the code point is above 0x10ffff.
pub(crate) fn encode_utf8(output: &mut [u8; 4], codepoint: u32) -> usize {
    match codepoint {
        0x0000..=0x007f => {
            output[0] = codepoint as u8;
            1
        }
        0x0080..=0x07ff => {
            output[0] = (((codepoint >> 6) & 0x1f) | 0xc0) as u8;
            output[1] = ((codepoint & 0x3f) | 0x80) as u8;
            2
        }
        0x0800..=0xffff => {
            output[0] = (((codepoint >> 12) & 0x0f) | 0xe0) as u8;
            output[1] = (((codepoint >> 6) & 0x3f) | 0x80) as u8;
            output[2] = ((codepoint & 0x3f) | 0x80) as u8;
            3
        }
        0x1_0000..=0x10_ffff => {
            output[0] = (((codepoint >> 18) & 0x07) | 0xf0) as u8;
            output[1] = (((codepoint >> 12) & 0x3f) | 0x80) as u8;
            output[2] = (((codepoint >> 6) & 0x3f) | 0x80) as u8;
            output[3] = ((codepoint & 0x3f) | 0x80) as u8;
            4
        }
        _ => 0,
    }
}

fn character_index_for_cluster(text: &str, cluster: u32) -> usize {
    let cluster = cluster as usize;
    text.char_indices()
        .take_while(|(byte_index, _)| *byte_index <= cluster)
        .count()
        .saturating_sub(1)
}
fn char_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod exact_utf_tests {
    use super::*;

    #[test]
    fn pinned_utf_owner_keeps_byte_level_non_scalar_behavior() {
        let bytes = [0xc0, 0x80, 0xed, 0xa0, 0x80, 0xf7, 0xbf, 0xbf, 0xbf];
        let mut offset = 0;
        assert_eq!(next_utf8(&bytes, &mut offset), Some(0));
        assert_eq!(next_utf8(&bytes, &mut offset), Some(0xd800));
        assert_eq!(next_utf8(&bytes, &mut offset), Some(0x1f_ffff));
        assert_eq!(offset, bytes.len());
        assert_eq!(count_utf8_length(&[0x80]), None);
    }

    #[test]
    fn pinned_utf16_length_and_encode_boundaries_are_preserved() {
        let mut utf16 = [0u16; 2];
        assert_eq!(to_utf16(0x1f600, &mut utf16), 2);
        assert_eq!(utf16, [0xd83d, 0xde00]);

        assert_eq!(
            count_code_point_length(&[0x7f, 0x80, 0x800, 0x1_0000, 0x11_0000]),
            10
        );
        let mut output = [0xaa; 4];
        assert_eq!(encode_utf8(&mut output, 0x10_ffff), 4);
        assert_eq!(output, [0xf4, 0x8f, 0xbf, 0xbf]);
        assert_eq!(encode_utf8(&mut output, 0x11_0000), 0);
        assert_eq!(output, [0xf4, 0x8f, 0xbf, 0xbf]);
    }
}
