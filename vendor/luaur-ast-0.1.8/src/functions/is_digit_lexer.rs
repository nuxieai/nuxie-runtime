#[allow(non_snake_case)]
#[inline]
pub fn is_digit(ch: char) -> bool {
    (ch as u8).wrapping_sub(b'0') < 10
}
