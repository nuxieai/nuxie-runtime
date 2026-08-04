#[allow(non_snake_case)]
#[inline]
pub fn is_alpha(ch: char) -> bool {
    // use or trick to convert to lower case and unsigned comparison to do range check
    ((ch as u8 | b' ').wrapping_sub(b'a')) < 26
}
