#[allow(non_snake_case)]
#[inline]
pub fn is_hex_digit(ch: char) -> bool {
    // use or trick to convert to lower case and unsigned comparison to do range check
    ((ch as u8).wrapping_sub(b'0') < 10) || (((ch as u8 | b' ').wrapping_sub(b'a')) < 6)
}
