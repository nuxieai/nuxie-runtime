pub fn u_posrelat(pos: i32, len: usize) -> i32 {
    if pos >= 0 {
        pos
    } else if (0usize).wrapping_sub(pos as usize) > len {
        // C: `0u - (size_t)pos > len` — negate POS, not len.
        0
    } else {
        len as i32 + pos + 1
    }
}
