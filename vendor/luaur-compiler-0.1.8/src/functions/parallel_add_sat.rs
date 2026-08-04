#[inline]
pub(crate) fn parallel_add_sat(x: u64, y: u64) -> u64 {
    let r = x.wrapping_add(y);
    let s = r & 0x8080808080808080u64; // saturation mask

    (r ^ s) | (s.wrapping_sub(s >> 7))
}
